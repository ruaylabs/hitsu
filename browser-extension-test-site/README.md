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

## Related-host scenario

The "Related-host isolation" scenario needs two sibling hostnames. Modern browsers resolve
`*.localhost` to the loopback, so the scenario works without extra setup; if yours does not,
add matching entries to `/etc/hosts`:

```
127.0.0.1 page.hitsu.localhost login.hitsu.localhost
```

1. Run the site as shown above (it serves every hostname it is reached on).
2. Create a Hitsu login with the URL `http://login.hitsu.localhost:8080`.
3. Open <http://page.hitsu.localhost:8080/scenario/related-host>.

Focusing the fields must not show inline suggestions, while the toolbar popup can still fill
the login, because the two hostnames share the `hitsu.localhost` registrable domain.
