# Browser-extension test site

Small, dependency-free Go web app for manually exercising Hitsu browser-extension autofill.

## Run

Go 1.22 or newer is required.

```sh
cd browser-extension-test-site
go run main.go
```

Open <http://localhost:8080>. Create a Hitsu login with that URL, then choose a scenario and use the extension toolbar popup to fill it.

Use a different loopback address if needed:

```sh
go run main.go -addr 127.0.0.1:9090
```
