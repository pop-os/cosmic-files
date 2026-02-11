cosmic-files = Pliki COSMIC
empty-folder = Pusty katalog
empty-folder-hidden = Pusty katalog (z ukrytymi plikami)
no-results = Brak wyników
filesystem = System plików
home = Katalog domowy
networks = Sieci
notification-in-progress = Operacje na plikach w toku
trash = Kosz
recents = Poprzednie
undo = Cofnij
today = Dzisiaj
# Desktop view options
desktop-view-options = Opcje widoku pulpitu…
show-on-desktop = Pokaż na Pulpicie
desktop-folder-content = Zawartość katalogu Pulpit
mounted-drives = Podpięte dyski
trash-folder-icon = Ikona kosza
icon-size-and-spacing = Rozmiar i rozstaw ikon
icon-size = Rozmiar ikon
grid-spacing = Rozstaw siatki
# List view
name = Nazwa
modified = Zmodyfikowano
trashed-on = Wyrzucono do kosza
size = Rozmiar
# Progress footer
details = Detale
dismiss = Odrzuć wiadomość
operations-running =
    { $running } bieżące { $running ->
        [one] działanie
       *[other] działania
    } ({ $percent }%)…
operations-running-finished =
    { $running } bieżące { $running ->
        [one] działanie
       *[other] działania
    } ({ $percent }%), { $finished } ukończone…
pause = Wstrzymaj
resume = Wznów

# Dialogs


## Compress Dialog

create-archive = Utwórz archiwum

## Extract Dialog

extract-password-required = Wymagane hasło
extract-to = Wypakuj do…
extract-to-title = Wypakuj do katalogu

## Empty Trash Dialog

empty-trash = Opróżnienie kosza
empty-trash-warning = Elementy z kosza zostaną bezpowrotnie usunięte

## Mount Error Dialog

mount-error = Brak dostępu do dysku
# New File/Folder Dialog
create-new-file = Utwórz nowy plik
create-new-folder = Utwórz nowy katalog
file-name = Nazwa pliku
folder-name = Nazwa katalogu
file-already-exists = Plik z taką nazwą już istnieje
folder-already-exists = Katalog z taką nazwą już istnieje
name-hidden = Nazwy zaczynające się od „.” będą ukryte
name-invalid = Musisz zmienić nazwę na inną z „{ $filename }”
name-no-slashes = Nazwa nie może zawierać ukośników
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

open-with-title = Czym chcesz otworzyć „{ $name }”?
browse-store = Przeglądaj { $store }
other-apps = Inne aplikacje
related-apps = Pokrewne aplikacje

## Permanently delete Dialog

selected-items = { $items } zaznaczonych elementów
permanently-delete-question = Definitywnie usunąć?
delete = Usuń
permanently-delete-warning = { $target } będzie bezpowrotnie usunięte. Nie będzie można tego przywrócić.
# Rename Dialog
rename-file = Zmień nazwę pliku
rename-folder = Zmień nazwę katalogu
# Replace Dialog
replace = Zastąp
replace-title = „{ $filename }” już istnieje w tym miejscu
replace-warning = Czy chcesz by został on zastąpiony przez wybrany element? To nadpisze jego zawartość.
replace-warning-operation = Czy chcesz by został on zastąpiony? To nadpisze jego zawartość.
original-file = Oryginalny plik
replace-with = Zastąpiony przez
apply-to-all = Zastosuj do wszystkich
keep-both = Zachowaj oba
skip = Pomiń

## Set as Executable and Launch Dialog

set-executable-and-launch = Ustaw jako wykonywalny i uruchom
set-executable-and-launch-description = Czy chcesz ustawić plik „{ $name }” jako wykonywalny i uruchomić go?
set-and-launch = Ustaw i uruchom

## Metadata Dialog

open-with = Otwórz za pomocą
owner = Właściciel
group = Grupa
other = Inni

### Mode 0

none = Brak

### Mode 1 (unusual)

execute-only = Tylko wykonywanie

### Mode 2 (unusual)

write-only = Tylko zapis

### Mode 3 (unusual)

write-execute = Zapis i wykonywanie

### Mode 4

read-only = Tylko odczyt

### Mode 5

read-execute = Odczyt i wykonywanie

### Mode 6

read-write = Odczyt i zapis

### Mode 7

read-write-execute = Odczyt, zapis i wykonywanie

## Favorite Path Error Dialog

favorite-path-error = Błąd podczas otwierania katalogu
favorite-path-error-description =
    Nie można otworzyć „{ $path }”
    Może nie istnieć lub możesz nie mieć uprawnień do jego otwierania

    Czy chcesz go usunąć z panelu bocznego?
remove = Usuń
keep = Zachowaj

# Context Pages


## About

repository = Repozytorium
support = Wsparcie

## Add Network Drive

add-network-drive = Dodaj dysk sieciowy
connect = Połącz
connect-anonymously = Połącz anonimowo
connecting = Łączenie…
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
    WebDAV,dav:// or davs://
network-drive-error = Brak dostępu do dysku sieciowego
password = Hasło
remember-password = Zapamiętaj hasło
try-again = Spróbuj ponownie
username = Nazwa użytkownika

## Operations

cancelled = Anulowano
edit-history = Historia edycji
history = Historia
no-history = Brak pozycji w historii.
pending = Oczekujące
progress = { $percent }%
progress-cancelled = { $percent }%, anulowano
progress-failed = { $percent }%, nieudane
progress-paused = { $percent }%, wstrzymano
failed = Nieudane
complete = Ukończone
compressing =
    Spakuj { $items } { $items ->
        [one] element
        [few] elementy
       *[other] elementów
    } z „{ $from }” do „{ $to }” ({ $progress })…
compressed =
    Spakowano { $items } { $items ->
        [one] element
        [few] elementy
       *[other] elementów
    } z „{ $from }” do „{ $to }”
copy_noun = Kopiuj
creating = Tworzy „{ $name }” w „{ $parent }”
created = Stworzono „{ $name }” w „{ $parent }”
copying =
    Kopiowanie { $items } { $items ->
        [one] elementu
       *[other] elementów
    } z „{ $from }” do „{ $to }” ({ $progress })…
copied =
    Skopiowano { $items } { $items ->
        [one] element
        [few] elementy
       *[other] elementów
    } z „{ $from }” do „{ $to }”
deleting =
    Usuwanie { $items } { $items ->
        [one] elementu
       *[other] elementów
    } z { trash } ({ $progress })...
deleted =
    Usunięto { $items } { $items ->
        [one] element
        [few] elementy
       *[other] elementów
    } z { trash }
emptying-trash = Opróżnianie { trash } ({ $progress })…
emptied-trash = Opróżniono { trash }
extracting =
    Wypakowywanie { $items } { $items ->
        [one] elementu
       *[other] elementów
    } z „{ $from }” do „{ $to }” ({ $progress })…
extracted =
    Wypakowano { $items } { $items ->
        [one] element
        [few] elementy
       *[other] elementów
    } z „{ $from }” do „{ $to }”
setting-executable-and-launching = Ustawianie „{ $name }” jako wykonywalnego i uruchamianie
set-executable-and-launched = Ustaw „{ $name }” jako wykonywalny i uruchom
setting-permissions = Ustawianie uprawnień dla „{ $name }” na { $mode }
set-permissions = Ustaw uprawnienia dla „{ $name }” na { $mode }
moving =
    Przenoszenie { $items } { $items ->
        [one] elementu
       *[other] elementów
    } z „{ $from }” do „{ $to }” ({ $progress })…
moved =
    Przeniesiono { $items } { $items ->
        [one] element
        [few] elementy
       *[other] elementów
    } z „{ $from }” do „{ $to }”
permanently-deleting =
    Definitywne usuwanie "{ $items }" "{ $items ->
        [one] elementu
       *[other] elementów
    }"
permanently-deleted =
    Definitywnie usunięto "{ $items }" "{ $items ->
        [one] element
        [few] elementy
       *[other] elementów
    }"
removing-from-recents =
    Usuwanie { $items } { $items ->
        [one] elementu
       *[other] elementów
    } z Poprzednich
removed-from-recents =
    Usunięto { $items } { $items ->
        [one] element
        [few] elementy
       *[other] elementów
    } z Poprzednich
renaming = Zmieniana nazwa z „{ $from }” na „{ $to }”
renamed = Zmieniono nazwę z „{ $from }” na „{ $to }”
restoring =
    Przywracanie { $items } { $items ->
        [one] elementu
       *[other] elementów
    } z Kosza ({ $progress })…
restored =
    Przywrócono { $items } { $items ->
        [one] element
        [few] elementy
       *[other] elementów
    } z Kosza
unknown-folder = nieznany katalog

## Open with

menu-open-with = Otwórz za pomocą…
default-app = { $name } (domyślnie)

## Show details

show-details = Pokaż szczegóły
type = Typ: { $mime }
items = Elementy: { $items }
item-size = Rozmiar: { $size }
item-created = Utworzono: { $created }
item-modified = Zmodyfikowano: { $modified }
item-accessed = Otwarto: { $accessed }
calculating = Obliczanie…

## Settings

settings = Ustawienia
single-click = Jedno kliknięcie by otwierać

### Appearance

appearance = Wygląd
theme = Motyw
match-desktop = Dopasuj do Pulpitu
dark = Ciemny
light = Jasny

### Type to Search

type-to-search = Zacznij pisać by wyszukać
type-to-search-recursive = Wyszukuj w obecnym katalogu i jego podkatalogach
type-to-search-enter-path = Wprowadź ścieżkę pliku lub katalogu
# Context menu
add-to-sidebar = Dodaj do bocznego panelu
compress = Spakuj
delete-permanently = Usuń definitywnie
eject = Wysuń
extract-here = Wypakuj
new-file = Nowy plik...
new-folder = Nowy katalog...
open-in-terminal = Otwórz w terminalu
move-to-trash = Przenieś do kosza
restore-from-trash = Przywróć z kosza
remove-from-sidebar = Usuń z bocznego panelu
sort-by-name = Uszereguj według nazwy
sort-by-modified = Uszereguj według czasu modyfikacji
sort-by-size = Uszereguj według rozmiaru
sort-by-trashed = Uszereguj według czasu usunięcia
remove-from-recents = Usuń z poprzednich

## Desktop

change-wallpaper = Zmień tapetę…
desktop-appearance = Wygląd pulpitu…
display-settings = Ustawienia wyświetlacza…

# Menu


## File

file = Plik
new-tab = Nowa karta
new-window = Nowe okno
reload-folder = Odśwież katalog
rename = Zmień nazwę…
close-tab = Zamknij kartę
quit = Zamknij

## Edit

edit = Edytuj
cut = Wytnij
copy = Kopiuj
paste = Wklej
select-all = Zaznacz wszystko

## View

zoom-in = Zbliż
default-size = Domyślny rozmiar
zoom-out = Oddal
view = Widok
grid-view = Widok siatki
list-view = Widok listy
show-hidden-files = Pokaż ukryte pliki
list-directories-first = Najpierw wyświetlaj katalogi
gallery-preview = Podgąd galerii
menu-settings = Ustawienia…
menu-about = O Plikach COSMIC…

## Sort

sort = Uszereguj
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Najpierw najnowsze
sort-oldest-first = Najpierw najstarsze
sort-smallest-to-largest = Najpierw najmniejsze
sort-largest-to-smallest = Najpierw największe
empty-trash-title = Opróżnić kosz?
type-to-search-select = Wybierz pierwszy pasujący plik lub katalog
pasted-image = Wklej Obraz
pasted-text = Wklejony Tekst
pasted-video = Wklejone Wideo
