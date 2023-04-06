# Wymagania
wszystkie wersje crate'ów zawarte są w Cargo.toml więc nie trzeba zwracać uwagi na instalację jakiejś specifycznej wersji

# Jak uruchomić API/panel administratorski

1. Zainstaluj rustup za pomocą instrukcji zamieszczonych pod tym linkiem
[instalacja rustup]( https://rustup.rs/)
2. Po instalacji zainstaluj najnowszą wersję rust za pomocą komendy
```
rustup install stable
```
3. Powyższa komenda zainstaluje także cargo
4. Załóż konto na stripe
5. Pobierz stripe-cli i uruchom komendę 
```
stripe listen --forward-to http://localhost:4765/api/payment/received
```
przekieruje ona pomyślne płatności na endpoint który doładowywuje konto
6. stwórz plik .env ze zmiennymi:
```
DATABASE_URL - URL bazy danych w formacie:
<silnik>://<nazwa użytkownika>:<hasło>@<host>/<nazwa bazy danych>
np.:
mysql://root:haslo@localhost/kantyna_app
EMAIL_PASS - hasło do emaila który będzie zażądzał wysyłaniem kodów (wymagany własny serwer SMTP)
EMAIL_NAME - nazwa maila od kodów
np. kantyna.noreply@mikut.dev
SMTP_RELAY - domena na której jest założony serwer SMTP
np. mikut.dev
STRIPE_SECRET - testowy token zapewniany przez stripe
np. sk_test_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
WEBHOOK_SECRET - token wygenerowany przez komendę z kroku 5.
JWT_SECRET - hash za pomocą którego JWT będzie szyfrowane, może być wygenerowany np. komendą openssl rand -base64 32
```
7. Stwórz bazę danych o nazwie podanej w DATABASE_URL
8. Za pomocą komendy 
```
cargo build
```
zbuduj cały program
9. Uruchom komendę
```
cargo run -- initdb
```
Zaimportuje ona wszystkie tabelki i pobierze aktualne menu ze strony elektronika
10. Znadując się w głównym katalogu (tam gdzie Cargo.toml) uruchom samą aplikację za pomocą
```
cargo run
```
wersja deweloperska
lub
```
cargo run --release
```
wersja produkcyjna
11. Na podomenach
```
http://127.0.0.1:4765/api/
```
ścieżki związane z api, udokumentowane w większości w pliku swagger.yaml w folderze docs/
Na domenie
```
http://127.0.0.1:4765/admin
```
Panel administratorski - wymaga on jednak stworzenia użytkownika i nadania mu admina (nadanie admina na razie jedynie możliwe za pomocą panelów bazodanowych np. phpmyadmin)
