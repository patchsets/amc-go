package fileio

import "time"

// Account is the representation of a combo loaded from a file.
type Account struct {
	Email    string
	Password string
}

// Config is the representation of the user defined settings to use
// when using this software.
type Config struct {
	Checker CfgChecker `json:"checker"`
	Proxy   CfgProxy   `json:"proxy_settings"`
}

// CfgChecker is the representation of checker settings inside the config.
type CfgChecker struct {
	// How many threads (goroutines) to use to check accounts concurrently.
	Threads int `json:"max_workers" default:"1"`
	// Whether or not to filter purchases on a working account.
	FilterGiftcards bool `json:"filter_giftcards" default:"false"`
	// Whether or not to capture purchases on a working account.
	DoFullCapture bool `json:"save_full_capture" default:"false"`
	// The limit of how many purchases should be captured.
}

// CfgCaptcha is the representation of captcha solver settings inside the config.
type CfgCaptcha struct {
	// Which captcha solving service to use.
	Service string `json:"service" default:"service"`
	// API Key for the captcha solver.
	APIKey string `json:"api_key" default:"api_key"`
	// Timeout for captcha solving service in milliseconds (ms), if applicable.
	Timeout int `json:"timeout" default:"2500"`
}

// CfgProxy is the representation of the proxy settings to use with the checker.
type CfgProxy struct {
	// The type of proxies to use while running.
	ProxyType string `json:"proxies_protocol" default:""`
	// The location of the proxy file to load from.
	ProxyFile string `json:"import_proxies_from" default:"proxies.txt"`
}

// Monitor controls the UI and saving valid accounts to their respective files.
type Monitor struct {
	// The path of the results directory to be used for saving accounts.
	ResultsPath string
	// The time at which the software started checking accounts.
	StartTime time.Time
	// The total number of hits found
	Hits int
	// The total number of errors that have occurred
	Errors int
	// The total number of creator accounts found
	Creators int
	// The total number of customs found
	Customs int
	// The total number of locked accounts found
	Locked int
	// The total number of accounts with errors
	ErrorAccs int
	// The total number of accounts tried with invalid login details
	Invalid int
	// The total number of accounts with cards linked
	WithCards int
	// The number of checks per minute.
	CPM int
	// The total number of accounts to check
	ComboLength int
	// The total number of captcha errors
	Retries int
	// The total number of accounts that have been checked
	Checked      int
	Progress     float64
	TotalBalance float64
}
