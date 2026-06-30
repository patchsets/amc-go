use anyhow::{bail, Context, Result};
use base64::Engine;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::process::Command;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Account {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub checker: CfgChecker,
    pub proxy_settings: CfgProxy,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CfgChecker {
    #[serde(default = "default_threads")]
    pub max_workers: usize,
    #[serde(default)]
    pub filter_giftcards: bool,
    #[serde(default)]
    pub save_full_capture: bool,
}

fn default_threads() -> usize {
    1
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CfgProxy {
    #[serde(default)]
    pub proxies_protocol: String,
    #[serde(default = "default_proxy_file")]
    pub import_proxies_from: String,
}

fn default_proxy_file() -> String {
    "proxies.txt".into()
}

pub struct Monitor {
    pub results_path: Mutex<String>,
    pub start_time: Instant,
    pub hits: AtomicU64,
    pub errors: AtomicU64,
    pub invalid: AtomicU64,
    pub with_cards: AtomicU64,
    pub retries: AtomicU64,
    pub checked: AtomicU64,
    pub combo_length: AtomicU64,
    pub total_balance: AtomicI64,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            results_path: Mutex::new(String::new()),
            start_time: Instant::now(),
            hits: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            invalid: AtomicU64::new(0),
            with_cards: AtomicU64::new(0),
            retries: AtomicU64::new(0),
            checked: AtomicU64::new(0),
            combo_length: AtomicU64::new(0),
            total_balance: AtomicI64::new(0),
        }
    }

    pub fn init_result_directory(&self) -> Result<()> {
        let now = chrono::Local::now().format("%Y-%m-%d %H-%M-%S").to_string();
        let path = format!("Results/{}", now);
        fs::create_dir_all(&path)?;
        *self.results_path.lock().unwrap() = path;
        Ok(())
    }

    pub fn save_account(&self, capture: &str, category: &str) -> Result<()> {
        let results_path = self.results_path.lock().unwrap().clone();
        let filepath = format!("{}/{}.txt", results_path, category);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filepath)?;
        writeln!(file, "{}", capture)?;
        Ok(())
    }

    pub fn add_balance(&self, amount: f64) {
        let cents = (amount * 100.0) as i64;
        self.total_balance.fetch_add(cents, Ordering::Relaxed);
    }

    pub fn get_total_balance(&self) -> f64 {
        self.total_balance.load(Ordering::Relaxed) as f64 / 100.0
    }

    pub fn get_title(&self) -> String {
        let checked = self.checked.load(Ordering::Relaxed);
        let combo_length = self.combo_length.load(Ordering::Relaxed);
        let hits = self.hits.load(Ordering::Relaxed);
        let invalid = self.invalid.load(Ordering::Relaxed);
        let with_cards = self.with_cards.load(Ordering::Relaxed);
        let retries = self.retries.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);
        let total_balance = self.get_total_balance();

        let total = hits + invalid;
        let elapsed = self.start_time.elapsed().as_secs().max(1);
        let cpm = if total > 0 {
            (total as f64 / elapsed as f64 * 60.0) as u64
        } else {
            0
        };
        let progress = if combo_length > 0 {
            (checked as f64 / combo_length as f64) * 100.0
        } else {
            0.0
        };

        format!(
            "StellarAMC - Checked: {}/{} ~ ({:.2}%) - Hits: {} - Invalids: {} - With Balance: {} - Total Balance: ${:.0} - Retries: {} - Errors: {} - CPM: {}",
            checked, combo_length, progress, hits, invalid, with_cards, total_balance, retries, errors, cpm
        )
    }
}

pub fn load_accounts(filepath: &str) -> Result<Vec<Account>> {
    let filepath = filepath.trim().trim_matches('"').trim_matches('\'');
    let file = fs::File::open(filepath).context("Failed to open combo file")?;
    let reader = BufReader::new(file);
    let mut accounts = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            accounts.push(Account {
                email: parts[0].to_string(),
                password: parts[1].to_string(),
            });
        }
    }

    Ok(accounts)
}

pub fn load_proxies(filepath: &str) -> Result<Vec<String>> {
    let filepath = filepath.trim().trim_matches('"').trim_matches('\'');
    let file = fs::File::open(filepath).context("Failed to open proxy file")?;
    let reader = BufReader::new(file);
    let mut proxies = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 {
            proxies.push(line);
        }
    }

    Ok(proxies)
}

pub fn load_config(filepath: &str) -> Result<Config> {
    let filepath = filepath.trim().trim_matches('"').trim_matches('\'');
    let contents = fs::read_to_string(filepath).context("Failed to read config file")?;
    let config: Config = serde_json::from_str(&contents)?;

    if config.proxy_settings.proxies_protocol.is_empty() {
        bail!("proxies_protocol cannot be left blank (HTTP/SOCKS4/SOCKS5)");
    }
    if config.proxy_settings.import_proxies_from.is_empty() {
        bail!("import_proxies_from cannot be left blank");
    }

    Ok(config)
}

pub fn generate_random_string(length: usize) -> String {
    let mut buf = vec![0u8; length];
    rand::fill(&mut buf[..]);
    base64::engine::general_purpose::STANDARD.encode(&buf)
}

pub fn clear_terminal() {
    if cfg!(windows) {
        let _ = Command::new("cmd").args(["/c", "cls"]).status();
    } else {
        let _ = Command::new("clear").status();
    }
}

pub fn splash_screen() {
    clear_terminal();
    let s = r#"
[38;2;83;37;116m [38;2;84;37;117m [38;2;85;38;119m [38;2;86;39;120m_[38;2;87;40;122m_[38;2;89;41;123m_[38;2;90;41;125m_[38;2;91;42;126m_[38;2;92;43;128m [38;2;93;44;129m_[38;2;95;45;131m_[38;2;96;46;132m [38;2;97;46;134m [38;2;98;47;135m [38;2;100;48;137m [38;2;101;49;138m [38;2;102;50;140m [38;2;103;50;141m [38;2;104;51;143m_[38;2;106;52;144m_[38;2;107;53;146m_[38;2;108;54;147m_[38;2;109;55;149m [38;2;111;55;150m [38;2;112;56;152m [38;2;113;57;153m [38;2;114;58;155m [38;2;115;59;156m [38;2;117;59;158m [38;2;118;60;159m [38;2;119;61;161m [38;2;120;62;162m [38;2;121;63;164m_[38;2;123;64;165m_[38;2;124;64;167m_[38;2;125;65;168m_[38;2;126;66;170m_[38;2;128;67;171m_[38;2;129;68;173m [38;2;130;68;174m [38;2;131;69;176m [38;2;132;70;177m [38;2;134;71;179m [38;2;135;72;180m [38;2;136;73;182m [38;2;137;73;183m [38;2;139;74;185m [38;2;140;75;186m [38;2;141;76;188m [38;2;142;77;189m [38;2;143;77;191m_[38;2;145;78;192m_[38;2;146;79;194m [38;2;147;80;195m [38;2;148;81;197m [38;2;150;82;199m [0m
[38;2;83;37;116m [38;2;84;37;117m [38;2;85;38;119m/[38;2;86;39;120m [38;2;87;40;122m_[38;2;89;41;123m_[38;2;90;41;125m_[38;2;91;42;126m/[38;2;92;43;128m/[38;2;93;44;129m [38;2;95;45;131m/[38;2;96;46;132m_[38;2;97;46;134m_[38;2;98;47;135m_[38;2;100;48;137m_[38;2;101;49;138m [38;2;102;50;140m [38;2;103;50;141m/[38;2;104;51;143m [38;2;106;52;144m/[38;2;107;53;146m [38;2;108;54;147m/[38;2;109;55;149m_[38;2;111;55;150m_[38;2;112;56;152m_[38;2;113;57;153m [38;2;114;58;155m_[38;2;115;59;156m_[38;2;117;59;158m_[38;2;118;60;159m_[38;2;119;61;161m_[38;2;120;62;162m/[38;2;121;63;164m_[38;2;123;64;165m [38;2;124;64;167m [38;2;125;65;168m_[38;2;126;66;170m_[38;2;128;67;171m/[38;2;129;68;173m_[38;2;130;68;174m_[38;2;131;69;176m_[38;2;132;70;177m [38;2;134;71;179m [38;2;135;72;180m_[38;2;136;73;182m_[38;2;137;73;183m_[38;2;139;74;185m_[38;2;140;75;186m [38;2;141;76;188m [38;2;142;77;189m/[38;2;143;77;191m [38;2;145;78;192m/[38;2;146;79;194m_[38;2;147;80;195m_[38;2;148;81;197m_[38;2;150;82;199m_[0m
[38;2;83;37;116m [38;2;84;37;117m [38;2;85;38;119m\[38;2;86;39;120m_[38;2;87;40;122m_[38;2;89;41;123m [38;2;90;41;125m\[38;2;91;42;126m/[38;2;92;43;128m [38;2;93;44;129m_[38;2;95;45;131m_[38;2;96;46;132m/[38;2;97;46;134m [38;2;98;47;135m_[38;2;100;48;137m [38;2;101;49;138m\[38;2;102;50;140m/[38;2;103;50;141m [38;2;104;51;143m/[38;2;106;52;144m [38;2;107;53;146m/[38;2;108;54;147m [38;2;109;55;149m_[38;2;111;55;150m_[38;2;112;56;152m [38;2;113;57;153m`[38;2;114;58;155m/[38;2;115;59;156m [38;2;117;59;158m_[38;2;118;60;159m_[38;2;119;61;161m_[38;2;120;62;162m/[38;2;121;63;164m/[38;2;123;64;165m [38;2;124;64;167m/[38;2;125;65;168m [38;2;126;66;170m/[38;2;128;67;171m [38;2;129;68;173m_[38;2;130;68;174m_[38;2;131;69;176m [38;2;132;70;177m\[38;2;134;71;179m/[38;2;135;72;180m [38;2;136;73;182m_[38;2;137;73;183m_[38;2;139;74;185m [38;2;140;75;186m\[38;2;141;76;188m/[38;2;142;77;189m [38;2;143;77;191m/[38;2;145;78;192m [38;2;146;79;194m_[38;2;147;80;195m_[38;2;148;81;197m_[38;2;150;82;199m/[0m
[38;2;83;37;116m [38;2;84;37;117m_[38;2;85;38;119m_[38;2;86;39;120m_[38;2;87;40;122m/[38;2;89;41;123m [38;2;90;41;125m/[38;2;91;42;126m [38;2;92;43;128m/[38;2;93;44;129m_[38;2;95;45;131m/[38;2;96;46;132m [38;2;97;46;134m [38;2;98;47;135m_[38;2;100;48;137m_[38;2;101;49;138m/[38;2;102;50;140m [38;2;103;50;141m/[38;2;104;51;143m [38;2;106;52;144m/[38;2;107;53;146m [38;2;108;54;147m/[38;2;109;55;149m_[38;2;111;55;150m/[38;2;112;56;152m [38;2;113;57;153m/[38;2;114;58;155m [38;2;115;59;156m/[38;2;117;59;158m [38;2;118;60;159m [38;2;119;61;161m [38;2;120;62;162m/[38;2;121;63;164m [38;2;123;64;165m/[38;2;124;64;167m [38;2;125;65;168m/[38;2;126;66;170m [38;2;128;67;171m/[38;2;129;68;173m_[38;2;130;68;174m/[38;2;131;69;176m [38;2;132;70;177m/[38;2;134;71;179m [38;2;135;72;180m/[38;2;136;73;182m_[38;2;137;73;183m/[38;2;139;74;185m [38;2;140;75;186m/[38;2;141;76;188m [38;2;142;77;189m([38;2;143;77;191m_[38;2;145;78;192m_[38;2;146;79;194m [38;2;147;80;195m [38;2;148;81;197m)[38;2;150;82;199m [0m
[38;2;83;37;116m/[38;2;84;37;117m_[38;2;85;38;119m_[38;2;86;39;120m_[38;2;87;40;122m_[38;2;89;41;123m/[38;2;90;41;125m\[38;2;91;42;126m_[38;2;92;43;128m_[38;2;93;44;129m/[38;2;95;45;131m\[38;2;96;46;132m_[38;2;97;46;134m_[38;2;98;47;135m_[38;2;100;48;137m/[38;2;101;49;138m_[38;2;102;50;140m/[38;2;103;50;141m_[38;2;104;51;143m/[38;2;106;52;144m\[38;2;107;53;146m_[38;2;108;54;147m_[38;2;109;55;149m,[38;2;111;55;150m_[38;2;112;56;152m/[38;2;113;57;153m_[38;2;114;58;155m/[38;2;115;59;156m [38;2;117;59;158m [38;2;118;60;159m [38;2;119;61;161m/[38;2;120;62;162m_[38;2;121;63;164m/[38;2;123;64;165m [38;2;124;64;167m [38;2;125;65;168m\[38;2;126;66;170m_[38;2;128;67;171m_[38;2;129;68;173m_[38;2;130;68;174m_[38;2;131;69;176m/[38;2;132;70;177m\[38;2;134;71;179m_[38;2;135;72;180m_[38;2;136;73;182m_[38;2;137;73;183m_[38;2;139;74;185m/[38;2;140;75;186m_[38;2;141;76;188m/[38;2;142;77;189m_[38;2;143;77;191m_[38;2;145;78;192m_[38;2;146;79;194m_[38;2;147;80;195m/[38;2;148;81;197m [38;2;150;82;199m [0m"#;
    println!("{}\n\n", s);
}

pub fn set_console_title(title: &str) {
    #[cfg(windows)]
    {
        let _ = Command::new("cmd")
            .args(["/c", &format!("title {}", title)])
            .status();
    }
}
