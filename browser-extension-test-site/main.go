package main

import (
	"embed"
	"flag"
	"html/template"
	"log"
	"net/http"
	"os"
	"sort"
	"time"
)

type scenario struct {
	Slug        string
	Name        string
	Description string
	Expected    string
}

var scenarios = map[string]scenario{
	"basic": {
		Slug:        "basic",
		Name:        "Basic login",
		Description: "A conventional email and password form without autocomplete hints.",
		Expected:    "Both fields fill and the password field receives focus.",
	},
	"autocomplete": {
		Slug:        "autocomplete",
		Name:        "Autocomplete hints",
		Description: "A login form with explicit username and current-password hints.",
		Expected:    "Both fields fill.",
	},
	"username-first": {
		Slug:        "username-first",
		Name:        "Two-step login",
		Description: "A username-first flow like Google. Fill with Hitsu, then select Continue.",
		Expected:    "The username fills first; after Continue, the new password field fills automatically.",
	},
	"delayed": {
		Slug:        "delayed",
		Name:        "Delayed form",
		Description: "The login form is inserted two seconds after this page loads. Trigger Hitsu before it appears.",
		Expected:    "Hitsu waits for the form and fills both fields when they appear.",
	},
	"signup": {
		Slug:        "signup",
		Name:        "Signup refusal",
		Description: "A registration form with new-password and confirmation fields.",
		Expected:    "No field fills.",
	},
	"change-password": {
		Slug:        "change-password",
		Name:        "Change password",
		Description: "A form containing current, new, and confirmation password fields.",
		Expected:    "The username and current password fill; both new-password fields remain empty.",
	},
	"hidden": {
		Slug:        "hidden",
		Name:        "Hidden fields",
		Description: "A hidden login form precedes a visible login form.",
		Expected:    "Only the visible form fills.",
	},
	"shadow-dom": {
		Slug:        "shadow-dom",
		Name:        "Nested shadow DOM",
		Description: "The login form lives inside two nested open shadow roots.",
		Expected:    "Both fields inside the shadow root fill.",
	},
	"iframes": {
		Slug:        "iframes",
		Name:        "Iframe origins",
		Description: "One login is in a same-origin frame and another is in a sandboxed, opaque-origin frame.",
		Expected:    "The same-origin frame fills. The sandboxed frame remains empty.",
	},
	"no-login": {
		Slug:        "no-login",
		Name:        "No login form",
		Description: "A page with an unrelated search field and no password field.",
		Expected:    "The search field remains empty and Hitsu reports that it could not fill the page.",
	},
}

type pageData struct {
	Scenario  scenario
	Scenarios []scenario
}

//go:embed templates
var templateFiles embed.FS

var (
	pageTemplate      = template.Must(template.ParseFS(templateFiles, "templates/page.html"))
	frameTemplate     = template.Must(template.ParseFS(templateFiles, "templates/frame.html"))
	scenarioTemplates = loadScenarioTemplates()
)

func loadScenarioTemplates() map[string]*template.Template {
	templates := make(map[string]*template.Template, len(scenarios))
	for slug := range scenarios {
		page := template.Must(pageTemplate.Clone())
		templates[slug] = template.Must(page.ParseFS(templateFiles, "templates/scenarios/"+slug+".html"))
	}
	return templates
}

func main() {
	addr := flag.String("addr", "localhost:8080", "HTTP listen address")
	flag.Parse()

	server := &http.Server{
		Addr:              *addr,
		Handler:           routes(),
		ReadHeaderTimeout: 5 * time.Second,
	}

	log.Printf("Hitsu extension test site: http://%s", *addr)
	if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
		log.New(os.Stderr, "server: ", 0).Fatal(err)
	}
}

func routes() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /", indexHandler)
	mux.HandleFunc("GET /scenario/{slug}", scenarioHandler)
	mux.HandleFunc("GET /frame/login", frameHandler)
	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		_, _ = w.Write([]byte("ok\n"))
	})
	return mux
}

func indexHandler(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path != "/" {
		http.NotFound(w, r)
		return
	}

	list := make([]scenario, 0, len(scenarios))
	for _, item := range scenarios {
		list = append(list, item)
	}
	sort.Slice(list, func(i, j int) bool { return list[i].Name < list[j].Name })
	renderPage(w, pageTemplate, pageData{Scenarios: list})
}

func scenarioHandler(w http.ResponseWriter, r *http.Request) {
	slug := r.PathValue("slug")
	item, ok := scenarios[slug]
	if !ok {
		http.NotFound(w, r)
		return
	}
	renderPage(w, scenarioTemplates[slug], pageData{Scenario: item})
}

func renderPage(w http.ResponseWriter, page *template.Template, data pageData) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.Header().Set("Cache-Control", "no-store")
	if err := page.Execute(w, data); err != nil {
		log.Printf("render page: %v", err)
	}
}

func frameHandler(w http.ResponseWriter, _ *http.Request) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.Header().Set("Cache-Control", "no-store")
	if err := frameTemplate.Execute(w, nil); err != nil {
		log.Printf("render frame: %v", err)
	}
}
