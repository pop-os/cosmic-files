cosmic-files = COSMIC Tiedostot
empty-folder = Tyhjä kansio
empty-folder-hidden = Tyhjä kansio (sisältää piilotettuja kohteita)
no-results = Ei tuloksia
filesystem = Tiedostojärjestelmä
home = Koti
networks = Verkkoyhteydet
notification-in-progress = Tiedostotoimintoja käynnissä.
trash = Roskakori
recents = Viimeaikaiset
undo = Peruuta viimeisin toiminto
today = Tänään

# Desktop view options

desktop-view-options = Työpöytänäkymän asetukset…
show-on-desktop = Näytä työpöydällä
desktop-folder-content = Työpöytäkansion sisältö
mounted-drives = Tiedostojärjestelmään liitetyt kovalevyt
trash-folder-icon = Roskakorikansion kuva
icon-size-and-spacing = Kuvan koko ja välitys
icon-size = Kuvan koko

# List view

name = Nimi
modified = Muokattu
trashed-on = Siirretty roskakoriin
size = Koko

# Dialogs

## Compress Dialog

create-archive = Luo arkisto

## Empty Trash Dialog

empty-trash = Tyhjennä roskakori
empty-trash-warning = Haluatko varmasti tyhjentää koko roskakorin pysyvästi?

## Mount Error Dialog

mount-error = Levy on saavuttamattomissa

## New File/Folder Dialog

create-new-file = Luo uusi tiedosto
create-new-folder = Luo uusi kansio
file-name = Tiedoston nimi
folder-name = Kansion nimi
file-already-exists = Annetun niminen tiedosto on jo olemassa.
folder-already-exists = Annetun niminen kansio on jo olemassa.
name-hidden = Merkillä "." alkavat nimet piilotetaan.
name-invalid = Nimi ei voi olla "{$filename}".
name-no-slashes = Nimi ei voi sisältää vinoviivoja.

## Open/Save Dialog

cancel = Peruuta
create = Luo
open = Avaa
open-file = Avaa tiedosto
open-folder = Avaa kansio
open-in-new-tab = Avaa uudessa välilehdessä
open-in-new-window = Avaa uudessa ikkunassa
open-item-location = Avaa kohteen sijainti
open-multiple-files = Avaa useita tiedostoja
open-multiple-folders = Avaa useita kansioita
save = Tallenna
save-file = Tallenna tiedosto

## Open With Dialog

open-with-title = Kuinka haluat avata kohteen "{$name}"?
browse-store = Selaa {$store}

## Rename Dialog

rename-file = Nimeä tiedosto uudelleen
rename-folder = Nimeä kansio uudelleen

## Replace Dialog

replace = Korvaa
replace-title = Kohde "{$filename}" on jo olemassa tässä sijainnissa.
replace-warning = Haluatko korvata sen tallentamallasi kohteella? Korvaaminen ylikirjoittaa kohteen sisällön.
replace-warning-operation = Haluatko korvata sen? Korvaaminen ylikirjoittaa sen sisällön.
original-file = Alkuperäinen tiedosto
replace-with = Korvaa kohteella
apply-to-all = Sovella kaikkiin
keep-both = Pidä molemmat
skip = Jätä välistä

## Set as Executable and Launch Dialog

set-executable-and-launch = Aseta käynnistettäväksi ja käynnistä
set-executable-and-launch-description = Haluatko asettaa kohteen "{$name}" käynnistettväksi ja käynnistää sen?
set-and-launch = Aseta ja käynnistä

## Metadata Dialog

owner = Omistaja
group = Ryhmä
other = Muut
read = Luku
write = Kirjoitus
execute = Käynnistys

# Context Pages

## About

git-description = Git-versio {$hash} päivänä {$date}

## Add Network Drive

add-network-drive = Lisää verkkolevy
connect = Yhdistä
connect-anonymously = Yhdistä nimettömästi
connecting = Yhdistää…
domain = Verkkotunnus
enter-server-address = Syötä palvelimen osoite
network-drive-description =
    Palvelinosoitteet sisältävät protokollaetuliitteen sekä osoitteen.
    Esimerkkejä: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Saatavissa olevat protokollat,Etuliite
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = Verkkolevy saavuttamattomissa
password = Salasana
remember-password = Muista salasana
try-again = Yritä uudelleen
username = Käyttäjänimi

## Operations

edit-history = Muokkaa historiaa
history = Historia
no-history = Historia on tyhjä.
pending = Odottaa käsittelyä
failed = Epäonnistui
complete = Valmis
compressing = Tiivistetään {$items} {$items ->
        [one] kohde
        *[other] kohteita
    } lähteestä "{$from}" arkistoon "{$to}"
compressed = Tiivistetty {$items} {$items ->
        [one] kohde
        *[other] kohteet
    } lähteestä "{$from}" arkistoon "{$to}"
copy_noun = Kopio
creating = Luodaan kohdetta "{$name}" kohteen "{$parent}" alle
created = Luotu kohde "{$name}" kohteen "{$parent}" alle
copying = Kopioidaan {$items} {$items ->
        [one] kohde
        *[other] kohteita
    } lähteestä "{$from}" kohteeseen "{$to}"
copied = Kopioitu {$items} {$items ->
        [one] kohde
        *[other] kohteet
    } lähteestä "{$from}" kohteeseen "{$to}"
emptying-trash = Tyhjennetään {trash}
emptied-trash = Tyhjennetty {trash}
extracting = Puretaan {$items} {$items ->
        [one] kohde
        *[other] kohteet
    } arkistosta "{$from}" kohteeseen "{$to}"
extracted = Purettu {$items} {$items ->
        [one] kohde
        *[other] kohteet
    } arkistosta "{$from}" kohteeseen "{$to}"
setting-executable-and-launching = Asetetaan "{$name}" käynnistettäväksi ja käynnistetään
set-executable-and-launched = Asetettu "{$name}" käynnistettäväksi ja käynnistetty
moving = Siirretään {$items} {$items ->
        [one] kohde
        *[other] kohteet
    } lähteestä "{$from}" kohteeseen "{$to}"
moved = Siirretty {$items} {$items ->
        [one] kohde
        *[other] kohteet
    } lähteestä "{$from}" kohteeseen "{$to}"
renaming = Nimetään kohde "{$from}" muotoon "{$to}"
renamed = Nimetty kohde "{$from}" muotoon "{$to}"
restoring = Palautetaan {$items} {$items ->
        [one] kohde
        *[other] kohteet
    } {trash}sta
restored = Palautettu {$items} {$items ->
        [one] kohde
        *[other] kohteet
    } {trash}sta
unknown-folder = Tuntematon kansio

## Open with
open-with = Avaa ohjelmalla…
default-app = {$name} (oletus)

## Show details
show-details = Näytä yksityiskohdat

## Settings
settings = Asetukset

### Appearance

appearance = Ulkoasu
theme = Teema
match-desktop = Sovita yhteen työpöydän kanssa
dark = Tumma
light = Vaalea

# Context menu

add-to-sidebar = Lisää sivupalkkiin
compress = Tiivistä
extract-here = Avaa tänne
new-file = Uusi tiedosto…
new-folder = Uusi kansio…
open-in-terminal = Avaa terminaalissa
move-to-trash = Siirrä roskakoriin
restore-from-trash = Palauta roskakorista
remove-from-sidebar = Poista sivupalkista
sort-by-name = Järjestä nimen mukaan
sort-by-modified = Järjestä muokkauspäivämäärän mukaan
sort-by-size = Järjestä koon mukaan
sort-by-trashed = Järjestä poistamispäivämäärän mukaan

## Desktop

change-wallpaper = Vaihda taustakuvaa…
desktop-appearance = Työpöydän ulkoasu…
display-settings = Näytön asetukset…

# Menu

## File

file = Tiedosto
new-tab = Uusi välilehti
new-window = Uusi ikkuna
rename = Nimeä uudelleen…
close-tab = Sulje välilehti
quit = Sulje

## Edit

edit = Muokkaa
cut = Leikkaa
copy = Kopioi
paste = Maalaa
select-all = Valitse kaikki

## View

zoom-in = Zoomaa sisään
default-size = Oletuskoko
zoom-out = Zoomaa ulos
view = Näkymä
grid-view = Ruudukkonäkymä
list-view = Listanäkymä
show-hidden-files = Näytä piilotetut tiedostot
list-directories-first = Näytä kansiot ensin
gallery-preview = Gallerian esinäkymä
menu-settings = Asetukset…
menu-about = Tietoa COSMIC Tiedostoista…

## Sort
sort = Järjestä
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Uusin ensin
sort-oldest-first = Vanhin ensin
sort-smallest-to-largest = Pienimmästä suurimpaan
sort-largest-to-smallest = Suurimmasta pienimpään
