cosmic-files = Pliki COSMIC
empty-folder = Pusty katalog
empty-folder-hidden = Pusty katalog (z ukrytymi plikami)
no-results = Brak wyników
filesystem = System plików
home = Katalog Domowy
networks = Sieci
notification-in-progress = Operacje na plikach w toku.
trash = Kosz
recents = Ubiegłe
undo = Cofnij
today = Dzisiaj

# Desktop view options
desktop-view-options = Opcje widoku pulpitu...
show-on-desktop = Pokaż na Pulpicie
desktop-folder-content = Zawartość katalogu Pulpit
mounted-drives = Podpięte dyski
trash-folder-icon = Ikona kosza
icon-size-and-spacing = Rozmiar i rozstaw ikon
icon-size = Rozmiar ikon

# List view
name = Nazwa
modified = Zmodyfikowano
trashed-on = Wyrzucono do kosza
size = Rozmiar

# Dialogs

## Compress Dialog
create-archive = Utwórz archiwum

## Empty Trash Dialog
empty-trash = Opróżnij kosz
empty-trash-warning = Czy chcesz bezpowrotnie usunąć zawartość Kosza?

# New File/Folder Dialog
create-new-file = Utwórz nowy plik
create-new-folder = Utwórz nowy katalog
file-name = Nazwa pliku
folder-name = Nazwa katalogu
file-already-exists = Plik z taką nazwą już istnieje.
folder-already-exists = Katalog z taką nazwą już istnieje.
name-hidden = Nazwy zaczynające się na "." będą ukryte.
name-invalid = Musisz zmienić nazwę na inną z "{$filename}".
name-no-slashes = Nazwa nie może zawierać ukośników.

# Open/Save Dialog
cancel = Anuluj
open = Otwórz
create = Utwórz
open-file = Otwórz plik
open-folder = Otwórz katalog
open-in-new-tab = Otwórz w nowej karcie
open-in-new-window = Otwórz w nowym oknie
open-item-location = Otwórz położenie elementu
open-multiple-files = Otwórz wiele plików
open-multiple-folders = Otwórz wiele katalogów
save = Zapisz
save-file = Zapisz plik

## Open With Dialog
open-with-title = Czym chcesz otworzyć "{$name}"?
browse-store = Przeglądaj {$store}

# Rename Dialog
rename-file = Zmień nazwę pliku
rename-folder = Zmień nazwę katalogu

# Replace Dialog
replace = Zastąp
replace-title = {$filename} już istnieje w tym miejscu.
replace-warning = Czy chcesz by został on zastąpiony? To nadpisze jego zawartość.
replace-warning-operation = Czy chcesz by został on zastąpiony? To nadpisze jego zawartość.
original-file = Oryginalny plik
replace-with = Zastąpiony przez
apply-to-all = Zastosuj do wszystkich
keep-both = Zachowaj oba
skip = Pomiń

## Set as Executable and Launch Dialog
set-executable-and-launch = Ustaw jako wykonywalny i uruchom
set-executable-and-launch-description = Czu chcesz ustawić "{$name}" jako wykonywalny i uruchomić?
set-and-launch = Ustaw i uruchom

## Metadata Dialog
owner = Właściciel
group = Grupa
other = Inni
read = Odczyt
write = Zapis
execute = Wykonywanie

# Context Pages

## About
git-description = Git commit {$hash} z {$date}

## Add Network Drive
add-network-drive = Dodaj dysk sieciowy
connect = Połącz
connect-anonymously = Połącz anonimowo
connecting = Łączenie...
domain = Domena
enter-server-address = Wprowadź adres serwera
network-drive-description =
    Adres serwera zawiera prefiks protokołu i adres.
    Przykładowo: ssh://192.168.0.1, ftp://[2001:db8::1]
### Make sure to keep the comma which separates the columns
network-drive-schemes =
    Dostępne protokoły,Prefiks
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = Brak dostępu do dysku sieciowego
password = Hasło
remember-password = Zapamiętaj hasło
try-again = Spróbuj ponownie
username = Nazwa użytkownika

## Operations
edit-history = Edytuj historię
history = Historia
no-history = Brak pozycji w historii.
pending = Oczekujące
failed = Nieudane
complete = Ukończone
compressing = Spakuj {$items} {$items ->
        [one] element
        [few] elementy
        *[other] elementów
    } z {$from} do {$to}
compressed = Spakowano {$items} {$items ->
        [one] element
        [few] elementy
        *[other] elementów
    } z {$from} do {$to}
copy_noun = Kopiuj
creating = Tworzy {$name} w {$parent}
created = Stworzono {$name} w {$parent}
copying = Kopiowanie {$items} {$items ->
        [one] elementu
        *[other] elementów
    } z {$from} do {$to}
copied = Skopiowano {$items} {$items ->
        [one] element
        [few] elementy
        *[other] elementów
    } z {$from} do {$to}
emptying-trash = Opróżnianie {trash}
emptied-trash = Opróżniono {trash}
extracting = Wypakowywanie {$items} {$items ->
        [one] elementu
        *[other] elementów
    } z {$from} do {$to}
extracted = Wypakowano {$items} {$items ->
        [one] element
        [few] elementy
        *[other] elementów
    } z {$from} do {$to}
moving = Przenoszenie {$items} {$items ->
        [one] elementu
        *[other] elementów
    } z {$from} do {$to}
moved = Przeniesiono {$items} {$items ->
        [one] element
        [few] elementy
        *[other] elementów
    } z {$from} do {$to}
renaming = Zmieniana nazwa {$from} na {$to}
renamed = Zmieniono nazwę {$from} na {$to}
restoring = Przywracanie {$items} {$items ->
        [one] elementu
        *[other] elementów
    } z {trash}
restored = Przywrócono {$items} {$items ->
        [one] element
        [few] elementy
        *[other] elementów
    } z {trash}
unknown-folder = nieznany katalog

## Open with
open-with = Otwórz za pomocą...
default-app = {$name} (domyślnie)

## Show details
show-details = Pokaż szczegóły

## Settings
settings = Ustawienia

### Appearance
appearance = Wygląd
theme = Motyw
match-desktop = Dopasuj do Pulpitu
dark = Ciemny
light = Jasny

# Context menu
extract-here = Wypakuj
add-to-sidebar = Dodaj do bocznego panelu
compress = Spakuj
extract-here = Wypakuj
new-file = Nowy plik
new-folder = Nowy katalog
open-in-terminal = Otwórz w terminalu
move-to-trash = Przenieś do kosza
restore-from-trash = Przywróć z kosza
remove-from-sidebar = Usuń z bocznego panelu
sort-by-name = Uszereguj według nazwy
sort-by-modified = Uszereguj według czasu modyfikacji
sort-by-size = Uszereguj według rozmiaru
sort-by-trashed = Uszereguj według czasu usunięcia

## Desktop
change-wallpaper = Zmień tapetę...
desktop-appearance = Wygląd pulpitu...
display-settings = Ustawienia wyświetlacza...

# Menu

## File
file = Plik
new-tab = Nowa karta
new-window = Nowe okno
rename = Zmień nazwę...
menu-show-details = Pokaż szczegóły...
close-tab = Zamknij kartę
quit = Zamknij

## Edit
edit = Edytuj
cut = Wytnij
copy = Kopiuj
paste = Wklej
select-all = Zaznacz wszystko

## View
zoom-in = Przybliż
default-size = Domyślny rozmiar
zoom-out = Oddal
view = Widok
grid-view = Widok siatki
list-view = Widok listy
show-hidden-files = Pokaż ukryte pliki
list-directories-first = Najpierw wyświetlaj katalogi
gallery-preview = Podgąd galerii
menu-settings = Ustawienia...
menu-about = O Plikach COSMIC...

## Sort
sort = Uszereguj
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Najpierw najnowsze
sort-oldest-first = Najpierw najstarsze
sort-smallest-to-largest = Najpierw najmniejsze
sort-largest-to-smallest = Najpierw największe
