package fileio

import (
	"bufio"
	"bytes"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"math"
	"os"
	"os/exec"
	"runtime"
	"strconv"
	"strings"
	"time"
	"unicode/utf8"

	title "github.com/lxi1400/GoTitle"
)

// Cleans the filepath.
// Sometimes the string is malformed if the file is dragged
// into the console/terminal window.
func cleanPath(filepath string) string {
	filepath = strings.Trim(filepath, " ")
	filepath = strings.ReplaceAll(filepath, "\"", "")
	filepath = strings.ReplaceAll(filepath, "'", "")

	return filepath
}

// LoadAccounts takes in a filepath as a string and returns an slice
// of Account structs. In case of an error, an empty slice is returned
// along with the corresponding error.
func LoadAccounts(filepath string) ([]Account, error) {
	filepath = cleanPath(filepath)

	file, err := os.Open(filepath)

	if err != nil {
		return []Account{}, err
	}

	accounts := []Account{}

	scanner := bufio.NewScanner(file)

	for scanner.Scan() {
		text := scanner.Text()

		if split_acc := strings.Split(text, ":"); len(split_acc) == 2 {
			accounts = append(accounts, Account{Email: split_acc[0], Password: split_acc[1]})
		}
	}

	return accounts, nil
}

// LoadProxies takes in a filepath and proxytype as strings and returns an slice of
// strings containing proxy addresses and ports, separated by a colon (:).
// In case of an error, an empty slice is returned along with the
// corresponding error.
func LoadProxies(filepath string) ([]string, error) {
	filepath = cleanPath(filepath)

	file, err := os.Open(filepath)

	if err != nil {
		return []string{}, err
	}

	proxies := []string{}

	scanner := bufio.NewScanner(file)

	for scanner.Scan() {
		text := scanner.Text()

		if len(strings.Split(text, ":")) >= 2 {
			proxies = append(proxies, text)
			// proxies = append(proxies, fmt.Sprintf("%s://%s", strings.ToLower(proxytype), text))
		}
	}

	return proxies, nil
}

// LoadUserAgents loads a list of useragents from 'useragents.txt'
// and returns them as a slice of strings. In case of an error,
// an empty slice is returned along with the corresponding error.
func LoadUserAgents() ([]string, error) {
	file, err := os.Open("./useragents.txt")

	if err != nil {
		return []string{}, err
	}

	scanner := bufio.NewScanner(file)

	userAgents := []string{}

	for scanner.Scan() {
		text := scanner.Text()
		userAgents = append(userAgents, text)

	}

	return userAgents, nil
}

// LoadConfig takes in a filepath as a string and returns a Config struct.
// It performs some checks to ensure that the config is valid.
// In case of an error, a blank config is returned along with the error.
func LoadConfig(filepath string) (Config, error) {
	filepath = cleanPath(filepath)

	file, err := os.Open(filepath)

	if err != nil {
		return Config{}, err
	}

	file_bytes, err := io.ReadAll(file)

	if err != nil {
		return Config{}, err
	}

	config := Config{}

	err = json.Unmarshal(file_bytes, &config)

	// Ensure proxy type & captcha service is uppercase.
	config.Proxy.ProxyType = strings.ToUpper(config.Proxy.ProxyType)

	if err != nil {
		return Config{}, err
	}

	// Specific JSON field error checking.

	if config.Proxy.ProxyType == "" {
		return Config{}, errors.New("proxy_type cannot be left blank (HTTP/SOCKS4/SOCKS5)")
	}

	if config.Proxy.ProxyFile == "" {
		return Config{}, errors.New("proxies_file cannot be left blank")
	}

	return config, nil
}

type Cycler[T any] struct {
	items []T
	index int
}

func NewCycler[T any](items []T) *Cycler[T] {
	return &Cycler[T]{items: items}
}

func (c *Cycler[T]) Next() T {
	val := c.items[c.index]
	c.index = (c.index + 1) % len(c.items)
	return val
}

func (c *Cycler[T]) Previous() T {
	c.index = (c.index - 1 + len(c.items)) % len(c.items)
	return c.items[c.index]
}

// InitResultDirectory will create a directory for the results, named using
// the date and time of when the program was launched. An error is returned,
// if applicable.
func (m *Monitor) InitResultDirectory() error {
	now := time.Now()
	now_formatted := now.Format("2006-01-02 15-04-05")

	err := os.MkdirAll(fmt.Sprintf("Results/%s", now_formatted), 0770)

	if err != nil {
		return err
	}
	m.ResultsPath = fmt.Sprintf("Results/%s", now_formatted)

	return nil
}

func (m *Monitor) SaveAccount(capture string, category string) error {
	file, err := os.OpenFile(fmt.Sprintf("%s/%s", m.ResultsPath, fmt.Sprintf("%s.txt", category)), os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0770)

	if err != nil {
		return err
	}
	file.WriteString(capture + "\n")

	file.Close()

	return nil
}

// ClearTerminal clears retrieves the running operating system
// and clears the terminal.
func ClearTerminal() {
	var cmd *exec.Cmd

	if runtime.GOOS == "windows" {
		cmd = exec.Command("cmd", "/c", "cls")
	} else {
		cmd = exec.Command("clear")
	}
	cmd.Stdout = os.Stdout
	cmd.Run()
}

// Center text takes in the width of the terminal, the string to print
// and returns a pointer to a buffer to be printed to the standard output.
func CenterText(width int, s string) *bytes.Buffer {
	const half, space = 2, "\u0020"
	var b bytes.Buffer
	n := (width - utf8.RuneCountInString(s)) / half
	if n < 0 {
		n = 0
	}
	fmt.Fprintf(&b, "%s%s", strings.Repeat(space, n), s)
	return &b
}

func GenerateRandomString(length int) string {
	b := make([]byte, length)
	_, err := rand.Read(b)
	if err != nil {
		panic(err)
	}
	return base64.StdEncoding.EncodeToString(b)
}

// SplashScreen clears the standard output and prints the banner to stdout
func SplashScreen() {
	ClearTerminal()

	var s string

	s += `
[38;2;83;37;116m [38;2;84;37;117m [38;2;85;38;119m [38;2;86;39;120m_[38;2;87;40;122m_[38;2;89;41;123m_[38;2;90;41;125m_[38;2;91;42;126m_[38;2;92;43;128m [38;2;93;44;129m_[38;2;95;45;131m_[38;2;96;46;132m [38;2;97;46;134m [38;2;98;47;135m [38;2;100;48;137m [38;2;101;49;138m [38;2;102;50;140m [38;2;103;50;141m [38;2;104;51;143m_[38;2;106;52;144m_[38;2;107;53;146m_[38;2;108;54;147m_[38;2;109;55;149m [38;2;111;55;150m [38;2;112;56;152m [38;2;113;57;153m [38;2;114;58;155m [38;2;115;59;156m [38;2;117;59;158m [38;2;118;60;159m [38;2;119;61;161m [38;2;120;62;162m [38;2;121;63;164m_[38;2;123;64;165m_[38;2;124;64;167m_[38;2;125;65;168m_[38;2;126;66;170m_[38;2;128;67;171m_[38;2;129;68;173m [38;2;130;68;174m [38;2;131;69;176m [38;2;132;70;177m [38;2;134;71;179m [38;2;135;72;180m [38;2;136;73;182m [38;2;137;73;183m [38;2;139;74;185m [38;2;140;75;186m [38;2;141;76;188m [38;2;142;77;189m [38;2;143;77;191m_[38;2;145;78;192m_[38;2;146;79;194m [38;2;147;80;195m [38;2;148;81;197m [38;2;150;82;199m [0m
[38;2;83;37;116m [38;2;84;37;117m [38;2;85;38;119m/[38;2;86;39;120m [38;2;87;40;122m_[38;2;89;41;123m_[38;2;90;41;125m_[38;2;91;42;126m/[38;2;92;43;128m/[38;2;93;44;129m [38;2;95;45;131m/[38;2;96;46;132m_[38;2;97;46;134m_[38;2;98;47;135m_[38;2;100;48;137m_[38;2;101;49;138m [38;2;102;50;140m [38;2;103;50;141m/[38;2;104;51;143m [38;2;106;52;144m/[38;2;107;53;146m [38;2;108;54;147m/[38;2;109;55;149m_[38;2;111;55;150m_[38;2;112;56;152m_[38;2;113;57;153m [38;2;114;58;155m_[38;2;115;59;156m_[38;2;117;59;158m_[38;2;118;60;159m_[38;2;119;61;161m_[38;2;120;62;162m/[38;2;121;63;164m_[38;2;123;64;165m [38;2;124;64;167m [38;2;125;65;168m_[38;2;126;66;170m_[38;2;128;67;171m/[38;2;129;68;173m_[38;2;130;68;174m_[38;2;131;69;176m_[38;2;132;70;177m [38;2;134;71;179m [38;2;135;72;180m_[38;2;136;73;182m_[38;2;137;73;183m_[38;2;139;74;185m_[38;2;140;75;186m [38;2;141;76;188m [38;2;142;77;189m/[38;2;143;77;191m [38;2;145;78;192m/[38;2;146;79;194m_[38;2;147;80;195m_[38;2;148;81;197m_[38;2;150;82;199m_[0m
[38;2;83;37;116m [38;2;84;37;117m [38;2;85;38;119m\[38;2;86;39;120m_[38;2;87;40;122m_[38;2;89;41;123m [38;2;90;41;125m\[38;2;91;42;126m/[38;2;92;43;128m [38;2;93;44;129m_[38;2;95;45;131m_[38;2;96;46;132m/[38;2;97;46;134m [38;2;98;47;135m_[38;2;100;48;137m [38;2;101;49;138m\[38;2;102;50;140m/[38;2;103;50;141m [38;2;104;51;143m/[38;2;106;52;144m [38;2;107;53;146m/[38;2;108;54;147m [38;2;109;55;149m_[38;2;111;55;150m_[38;2;112;56;152m [38;2;113;57;153m` + "`" + `[38;2;114;58;155m/[38;2;115;59;156m [38;2;117;59;158m_[38;2;118;60;159m_[38;2;119;61;161m_[38;2;120;62;162m/[38;2;121;63;164m/[38;2;123;64;165m [38;2;124;64;167m/[38;2;125;65;168m [38;2;126;66;170m/[38;2;128;67;171m [38;2;129;68;173m_[38;2;130;68;174m_[38;2;131;69;176m [38;2;132;70;177m\[38;2;134;71;179m/[38;2;135;72;180m [38;2;136;73;182m_[38;2;137;73;183m_[38;2;139;74;185m [38;2;140;75;186m\[38;2;141;76;188m/[38;2;142;77;189m [38;2;143;77;191m/[38;2;145;78;192m [38;2;146;79;194m_[38;2;147;80;195m_[38;2;148;81;197m_[38;2;150;82;199m/[0m
[38;2;83;37;116m [38;2;84;37;117m_[38;2;85;38;119m_[38;2;86;39;120m_[38;2;87;40;122m/[38;2;89;41;123m [38;2;90;41;125m/[38;2;91;42;126m [38;2;92;43;128m/[38;2;93;44;129m_[38;2;95;45;131m/[38;2;96;46;132m [38;2;97;46;134m [38;2;98;47;135m_[38;2;100;48;137m_[38;2;101;49;138m/[38;2;102;50;140m [38;2;103;50;141m/[38;2;104;51;143m [38;2;106;52;144m/[38;2;107;53;146m [38;2;108;54;147m/[38;2;109;55;149m_[38;2;111;55;150m/[38;2;112;56;152m [38;2;113;57;153m/[38;2;114;58;155m [38;2;115;59;156m/[38;2;117;59;158m [38;2;118;60;159m [38;2;119;61;161m [38;2;120;62;162m/[38;2;121;63;164m [38;2;123;64;165m/[38;2;124;64;167m [38;2;125;65;168m/[38;2;126;66;170m [38;2;128;67;171m/[38;2;129;68;173m_[38;2;130;68;174m/[38;2;131;69;176m [38;2;132;70;177m/[38;2;134;71;179m [38;2;135;72;180m/[38;2;136;73;182m_[38;2;137;73;183m/[38;2;139;74;185m [38;2;140;75;186m/[38;2;141;76;188m [38;2;142;77;189m([38;2;143;77;191m_[38;2;145;78;192m_[38;2;146;79;194m [38;2;147;80;195m [38;2;148;81;197m)[38;2;150;82;199m [0m
[38;2;83;37;116m/[38;2;84;37;117m_[38;2;85;38;119m_[38;2;86;39;120m_[38;2;87;40;122m_[38;2;89;41;123m/[38;2;90;41;125m\[38;2;91;42;126m_[38;2;92;43;128m_[38;2;93;44;129m/[38;2;95;45;131m\[38;2;96;46;132m_[38;2;97;46;134m_[38;2;98;47;135m_[38;2;100;48;137m/[38;2;101;49;138m_[38;2;102;50;140m/[38;2;103;50;141m_[38;2;104;51;143m/[38;2;106;52;144m\[38;2;107;53;146m_[38;2;108;54;147m_[38;2;109;55;149m,[38;2;111;55;150m_[38;2;112;56;152m/[38;2;113;57;153m_[38;2;114;58;155m/[38;2;115;59;156m [38;2;117;59;158m [38;2;118;60;159m [38;2;119;61;161m/[38;2;120;62;162m_[38;2;121;63;164m/[38;2;123;64;165m [38;2;124;64;167m [38;2;125;65;168m\[38;2;126;66;170m_[38;2;128;67;171m_[38;2;129;68;173m_[38;2;130;68;174m_[38;2;131;69;176m/[38;2;132;70;177m\[38;2;134;71;179m_[38;2;135;72;180m_[38;2;136;73;182m_[38;2;137;73;183m_[38;2;139;74;185m/[38;2;140;75;186m_[38;2;141;76;188m/[38;2;142;77;189m_[38;2;143;77;191m_[38;2;145;78;192m_[38;2;146;79;194m_[38;2;147;80;195m/[38;2;148;81;197m [38;2;150;82;199m [0m`
	s += "\n\n\n"
	fmt.Println(s)

}

func (m *Monitor) UpdateTitle() {
	if (m.Hits + m.Invalid) != 0 {
		ttl := float64(m.Hits + m.Invalid)
		cpm := float64(time.Now().Unix() - m.StartTime.Unix())
		m.CPM = int(math.Round(ttl/cpm)) * 60
		m.Progress = (float64(m.Checked) / float64(m.ComboLength)) * float64(100.00)
	} else {
		m.CPM = 0
		m.Progress = 0
	}

	prog := strconv.FormatFloat(m.Progress, 'f', 2, 64)
	text := fmt.Sprintf("StellarAMC - Checked: %d/%d ~ (%s", m.Checked, m.ComboLength, prog) + "%)" + fmt.Sprintf(" - Hits: %d - Invalids: %d - With Balance: %d - Total Balance: $%2.f - Retries: %d - Errors: %d - CPM: %d", m.Hits, m.Invalid, m.WithCards, m.TotalBalance, m.Retries, m.Errors, m.CPM)
	title.SetTitle(text)
}

func UpdateTitleExplicit(title_str string) {
	title.SetTitle(title_str)
}
