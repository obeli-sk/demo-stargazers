package main

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"strconv"
	"strings"

	"github.com/julienschmidt/httprouter"
	"go.bytecodealliance.org/cm"
	"go.wasmcloud.dev/component/net/wasihttp"

	logbindings "github.com/obeli-sk/demo-stargazers/webhook-go/gen/obelisk/log/log"
	timebindings "github.com/obeli-sk/demo-stargazers/webhook-go/gen/obelisk/types/time"
	dbbindings "github.com/obeli-sk/demo-stargazers/webhook-go/gen/stargazers/db/user"
	workflowbindings "github.com/obeli-sk/demo-stargazers/webhook-go/gen/stargazers/workflow-obelisk-schedule/workflow"
)

const (
	httpHeaderSignature            = "X-Hub-Signature-256"
	envGithubWebhookInsecure       = "GITHUB_WEBHOOK_INSECURE"
	envGithubWebhookSecret         = "GITHUB_WEBHOOK_SECRET"
	maxQueryLimit            uint8 = 5
)

type Action string

const (
	ActionCreated Action = "created"
	ActionDeleted Action = "deleted"
)

type User struct {
	Login string `json:"login"`
}

func (u User) String() string {
	return u.Login
}

type Repository struct {
	Name  string `json:"name"`
	Owner User   `json:"owner"`
}

func (r Repository) String() string {
	return fmt.Sprintf("%s/%s", r.Owner.Login, r.Name)
}

type StarEvent struct {
	Action     Action     `json:"action"`
	Sender     User       `json:"sender"`
	Repository Repository `json:"repository"`
}

// JSONStargazer is a struct tailored for JSON output.
// It uses *string for optional fields, which serializes to the string value or null.
type JSONStargazer struct {
	Login       string  `json:"login"`
	Repo        string  `json:"repo"`
	Description *string `json:"description,omitempty"`
}

// --- HTTP Handlers ---

func init() {
	router := httprouter.New()
	router.HandlerFunc(http.MethodPost, "/", postHandler)
	router.HandlerFunc(http.MethodGet, "/", getHandler)
	// Other methods will automatically result in a 405 Method Not Allowed from httprouter
	wasihttp.Handle(router)
}

// postHandler handles webhook events (Method::Post)
func postHandler(w http.ResponseWriter, r *http.Request) {
	sha256Signature := r.Header.Get(httpHeaderSignature)

	body, err := io.ReadAll(r.Body)
	if err != nil {
		fmt.Printf("Error reading request body: %v\n", err)
		http.Error(w, "Failed to read request body", http.StatusInternalServerError)
		return
	}
	defer r.Body.Close()

	if os.Getenv(envGithubWebhookInsecure) == "true" {
		fmt.Printf("WARN: Not verifying the request because %s is set to `true`!\n", envGithubWebhookInsecure)
	} else {
		secret := os.Getenv(envGithubWebhookSecret)
		if secret == "" {
			panic(fmt.Sprintf("%s must be passed as environment variable", envGithubWebhookSecret))
		}
		if sha256Signature == "" {
			panic(fmt.Sprintf("HTTP header %s must be set", httpHeaderSignature))
		}
		// verifySignature will panic on failure
		verifySignature(secret, body, sha256Signature)
	}

	var event StarEvent
	if err := json.Unmarshal(body, &event); err != nil {
		fmt.Printf("Cannot deserialize payload - %v\n", err)
		http.Error(w, "Cannot deserialize payload", http.StatusBadRequest)
		return
	}

	fmt.Printf("Got event: %+v\n", event)
	repoFullName := event.Repository.String()

	scheduleAt := timebindings.ScheduleAtNow()

	var executionID workflowbindings.ExecutionID
	switch event.Action {
	case ActionCreated:
		// WIT: star-added-schedule: func(schedule-at: schedule-at, login: string, repo: string) -> execution-id;
		executionID = workflowbindings.StarAddedSchedule(scheduleAt, event.Sender.Login, repoFullName)
	case ActionDeleted:
		// WIT: star-removed-schedule: func(schedule-at: schedule-at, login: string, repo: string) -> execution-id;
		executionID = workflowbindings.StarRemovedSchedule(scheduleAt, event.Sender.Login, repoFullName)
	default:
		fmt.Printf("Unknown action: %s\n", event.Action)
		http.Error(w, fmt.Sprintf("Unknown action: %s", event.Action), http.StatusBadRequest)
		return
	}

	w.Header().Set("execution-id", executionID.ID)
	w.WriteHeader(http.StatusOK)
}

// verifySignature verifies a message using a shared secret and X-Hub-Signature-256.
// It panics on failure
func verifySignature(secret string, payload []byte, hubSignature string) {
	if !strings.HasPrefix(hubSignature, "sha256=") {
		panic(fmt.Sprintf("HTTP header %s must start with `sha256=`", httpHeaderSignature))
	}
	signatureHex := strings.TrimPrefix(hubSignature, "sha256=")

	signatureBytes, err := hex.DecodeString(signatureHex)
	if err != nil {
		panic(fmt.Sprintf("HTTP header %s must be hex-encoded: %v", httpHeaderSignature, err))
	}

	mac := hmac.New(sha256.New, []byte(secret))
	// mac.Write never returns an error for hash.Hash
	_, _ = mac.Write(payload)
	expectedMAC := mac.Sum(nil)

	if !hmac.Equal(signatureBytes, expectedMAC) {
		panic("Signature verification failed: calculated MAC does not match provided signature")
	}
	fmt.Println("Signature verified successfully.")
}

// getHandler renders a table with last few stargazers (Method::Get)
func getHandler(w http.ResponseWriter, r *http.Request) {
	query := r.URL.Query()

	limitStr := query.Get("limit")
	limit := maxQueryLimit // Default
	if limitStr != "" {
		parsedLimit, err := strconv.ParseUint(limitStr, 10, 8)
		if err == nil {
			limit = uint8(parsedLimit)
			if limit > maxQueryLimit {
				limit = maxQueryLimit
			}
		} else {
			fmt.Printf("Invalid limit query parameter '%s': %v. Using default %d.\n", limitStr, err, maxQueryLimit)
		}
	}

	repoQuery := query.Get("repo")
	var actualRepoArg cm.Option[string]
	if repoQuery != "" {
		actualRepoArg = cm.Some(repoQuery)
	} else {
		actualRepoArg = cm.None[string]()
	}

	dbOrdering := dbbindings.OrderingDescending
	if query.Get("ordering") == "asc" {
		dbOrdering = dbbindings.OrderingAscending
	}

	// WIT: stargazers:db/user.list-stargazers: func(last: u8, repo: option<string>, ordering: ordering) -> result<list<stargazer>, string>
	result := dbbindings.ListStargazers(limit, actualRepoArg, dbOrdering)

	if result.IsErr() {
		errMsg := result.Err()
		fmt.Printf("Error from list-stargazers: %s\n", errMsg)
		http.Error(w, "Failed to retrieve stargazer list", http.StatusInternalServerError)
		return
	}

	witStargazerList := result.OK().Slice()

	// Convert to JSON-friendly structs
	jsonStargazerList := make([]JSONStargazer, len(witStargazerList))
	for i, s := range witStargazerList {
		jsonStargazerList[i].Login = s.Login
		jsonStargazerList[i].Repo = s.Repo
		// Some returns a non-nil *T if o represents the some case,
		// or nil if o represents the none case.
		jsonStargazerList[i].Description = s.Description.Some()
	}

	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(jsonStargazerList); err != nil {
		fmt.Printf("Error encoding JSON response: %v\n", err)
		http.Error(w, "Failed to encode response", http.StatusInternalServerError)
	}
}

func main() {
	logbindings.Error("wasi:cli/run@0.2.0.run is exported only because of current TinyGo limitation")
	panic("wasi:cli/run@0.2.0.run is exported only because of current TinyGo limitation")
}
