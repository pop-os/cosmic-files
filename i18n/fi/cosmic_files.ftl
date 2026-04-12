cosmic-files = COSMICin tiedostot
empty-folder = Tyhjä kansio
empty-folder-hidden = Tyhjä kansio (sisältää piilotettuja kohteita)
no-results = Ei tuloksia
filesystem = Tiedostojärjestelmä
home = Koti
networks = Verkot
notification-in-progress = Tiedostotoimintoja käynnissä
trash = Roskakori
recents = Viimeaikaiset
undo = Kumoa
today = Tänään

# Desktop view options

desktop-view-options = Työpöytänäkymän asetukset…
show-on-desktop = Näytä työpöydällä
desktop-folder-content = Työpöytäkansion sisältö
mounted-drives = Liitetyt asemat
trash-folder-icon = Roskakorikansion kuvake
icon-size-and-spacing = Kuvakkeen koko ja välistys
icon-size = Kuvakkeen koko

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
empty-trash-warning = Roskakorikansion kohteet poistetaan pysyvästi

## Mount Error Dialog

mount-error = Levy on saavuttamattomissa

## New File/Folder Dialog

create-new-file = Luo uusi tiedosto
create-new-folder = Luo uusi kansio
file-name = Tiedoston nimi
folder-name = Kansion nimi
file-already-exists = Tiedosto samalla nimellä on jo olemassa
folder-already-exists = Kansio samalla nimellä on jo olemassa
name-hidden = Merkillä "." alkavat nimet piilotetaan
name-invalid = Nimi ei voi olla "{ $filename }"
name-no-slashes = Nimi ei voi sisältää vinoviivoja

## Open/Save Dialog

cancel = Peru
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

open-with-title = Miten haluat avata kohteen "{ $name }"?
browse-store = Selaa { $store }a

## Rename Dialog

rename-file = Nimeä tiedosto uudelleen
rename-folder = Nimeä kansio uudelleen

## Replace Dialog

replace = Korvaa
replace-title = "{ $filename }" on jo olemassa tässä sijainnissa
replace-warning = Haluatko korvata sen tallentamallasi kohteella? Korvaaminen ylikirjoittaa kohteen sisällön.
replace-warning-operation = Haluatko korvata sen? Korvaaminen ylikirjoittaa sen sisällön.
original-file = Alkuperäinen tiedosto
replace-with = Korvaa käyttäen
apply-to-all = Toteuta kaikkiin
keep-both = Pidä molemmat
skip = Ohita

## Set as Executable and Launch Dialog

set-executable-and-launch = Aseta käynnistettäväksi ja käynnistä
set-executable-and-launch-description = Haluatko asettaa kohteen "{ $name }" käynnistettäväksi ja käynnistää sen?
set-and-launch = Aseta ja käynnistä

## Metadata Dialog

owner = Omistaja
group = Ryhmä
other = Muut

# Context Pages


## About


## Add Network Drive

add-network-drive = Lisää verkkolevy
connect = Yhdistä
connect-anonymously = Yhdistä nimettömästi
connecting = Yhdistetään…
domain = Verkkotunnus
enter-server-address = Kirjoita palvelimen osoite
network-drive-description =
    Palvelinosoitteet sisältävät protokollaetuliitteen sekä osoitteen.
    Esimerkkejä: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Saatavilla olevat yhteyskäytännöt,Etuliite
    AppleTalk,afp://
    File Transfer Protocol,ftp:// tai ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// tai ssh://
    WebDav,dav:// tai davs://
network-drive-error = Verkkolevy ei saatavilla
password = Salasana
remember-password = Muista salasana
try-again = Yritä uudelleen
username = Käyttäjätunnus

## Operations

edit-history = Muokkaa historiaa
history = Historia
no-history = Historia on tyhjä.
pending = Jonossa
failed = Epäonnistuneet
complete = Valmiit
compressing =
    Pakataan { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } sijainnista "{ $from }" arkistoon "{ $to }" ({ $progress })…
compressed =
    Pakattu { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } sijainnista "{ $from }" arkistoon "{ $to }"
copy_noun = Kopio
creating = Luodaan "{ $name }" kohteen "{ $parent }" alle
created = Luotu "{ $name }" kohteen "{ $parent }" alle
copying =
    Kopioidaan { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } sijainnista "{ $from }" kohteeseen "{ $to }" ({ $progress })…
copied =
    Kopioitu { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } sijainnista "{ $from }" kohteeseen "{ $to }"
emptying-trash = Tyhjennetään { trash } ({ $progress })…
emptied-trash = Tyhjennetty { trash }
extracting =
    Puretaan { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } arkistosta "{ $from }" kohteeseen "{ $to }" ({ $progress })…
extracted =
    Purettu { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } arkistosta "{ $from }" kohteeseen "{ $to }"
setting-executable-and-launching = Asetetaan "{ $name }" käynnistettäväksi ja käynnistetään
set-executable-and-launched = Asetettu "{ $name }" käynnistettäväksi ja käynnistetty
moving =
    Siirretään { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } sijainnista "{ $from }" kohteeseen "{ $to }" ({ $progress })…
moved =
    Siirretty { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } sijainnista "{ $from }" kohteeseen "{ $to }"
renaming = Nimetään kohde "{ $from }" muotoon "{ $to }"
renamed = Nimetty kohde "{ $from }" muotoon "{ $to }"
restoring =
    Palautetaan { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } roskakorista ({ $progress })…
restored =
    Palautettu { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } roskakorista
unknown-folder = tuntematon kansio

## Open with

menu-open-with = Avaa sovelluksella…
default-app = { $name } (oletus)

## Show details

show-details = Näytä yksityiskohdat

## Settings

settings = Asetukset

### Appearance

appearance = Ulkoasu
theme = Teema
match-desktop = Sovita työpöytään
dark = Tumma
light = Vaalea

# Context menu

add-to-sidebar = Lisää sivupalkkiin
compress = Pakkaa…
extract-here = Pura
new-file = Uusi tiedosto…
new-folder = Uusi kansio…
open-in-terminal = Avaa päätteessä
move-to-trash = Siirrä roskakoriin
restore-from-trash = Palauta roskakorista
remove-from-sidebar = Poista sivupalkista
sort-by-name = Järjestä nimen mukaan
sort-by-modified = Järjestä muokkausajan mukaan
sort-by-size = Järjestä koon mukaan
sort-by-trashed = Järjestä poistamisajan mukaan

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
quit = Lopeta

## Edit

edit = Muokkaa
cut = Leikkaa
copy = Kopioi
paste = Liitä
select-all = Valitse kaikki

## View

zoom-in = Lähennä
default-size = Oletuskoko
zoom-out = Loitonna
view = Näytä
grid-view = Ruudukkonäkymä
list-view = Listanäkymä
show-hidden-files = Näytä piilotetut tiedostot
list-directories-first = Näytä kansiot ensin
gallery-preview = Gallerian esikatselu
menu-settings = Asetukset…
menu-about = Tietoa COSMICin tiedostonhallinnasta…

## Sort

sort = Järjestä
sort-a-z = A-Ö
sort-z-a = Ö-A
sort-newest-first = Uusin ensin
sort-oldest-first = Vanhin ensin
sort-smallest-to-largest = Pienimmästä suurimpaan
sort-largest-to-smallest = Suurimmasta pienimpään
resume = Jatka
extract-password-required = Salasana vaaditaan
extract-to-title = Pura kansioon
empty-trash-title = Tyhjennetäänkö roskakori?
other-apps = Muut sovellukset
related-apps = Liittyvät sovellukset
permanently-delete-question = Poistetaanko pysyvästi?
delete = Poista
open-with = Avaa sovelluksella
remove = Poista
cancelled = Peruttu
type = Tyyppi: { $mime }
item-size = Koko: { $size }
item-created = Luotu: { $created }
item-modified = Muokattu: { $modified }
delete-permanently = Poista pysyvästi
reload-folder = Lataa kansio uudelleen
comment = Tiedostonhallinta COSMIC-työpöydälle
keywords = Folder;Manager;Kansio;Hakemisto;Hallinta;Hallinnointi;Hallitse;Hallinnoi;
copy-to-button-label = Kopioi
move-to-button-label = Siirrä
clear-recents-history = Tyhjennä viimeaikaisten historia
copy-path = Kopioi polku
dismiss = Hylkää viesti
operations-running =
    { $running } { $running ->
        [one] toiminto
       *[other] toimintoa
    } käynnissä ({ $percent } %)...
operations-running-finished =
    { $running } { $running ->
        [one] toiminto
       *[other] toimintoa
    } käynnissä ({ $percent } %), { $finished } valmistunut…
pause = Keskeytä
extract-to = Pura sijaintiin…
permanently-delete-warning = { $target } tullaan poistamaan pysyvästi. Tätä toimintoa ei voi perua.
execute-only = Vain suoritus
write-only = Vain kirjoitus
write-execute = Kirjoita ja suorita
read-only = Vain luku
read-execute = Lue ja suorita
read-write = Lue ja kirjoita
read-write-execute = Lue, kirjoita sekä suorita
calculating = Lasketaan…
single-click = Yhden napsautuksen avaus
type-to-search = Kirjoita etsiäksesi
type-to-search-recursive = Etsii nykyisestä kansiosta ja kaikista alikansioista
remove-from-recents = Poista viimeaikaisista
selected-items = { $items } valittua kohdetta
show-recents = Viimeaikaisten kansio sivupalkissa
copy-to = Kopioi…
move-to = Siirrä…
details = Yksityiskohdat
grid-spacing = Ruudukkovälit
none = Ei mitään
favorite-path-error = Virhe avattaessa kansiota
favorite-path-error-description =
    Polun { $path } avaaminen ei onnistunut
    "{ $path }" ei välttämättä ole olemassa tai oikeutesi eivät riitä sen avaamiseen

    Haluatko poistaa sen sivupalkista?
keep = Pidä
repository = Tietovarasto
support = Tuki
progress = { $percent } %
progress-cancelled = { $percent } %, peruttu
progress-failed = { $percent } %, epäonnistui
progress-paused = { $percent } %, keskeytetty
setting-permissions = Asetetaan kohteen "{ $name }" käyttöoikeudeksi { $mode }
set-permissions = Asetettu kohteen { $name } käyttöoikeudeksi { $mode }
permanently-deleting =
    Poistetaan pysyvästi { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    }
permanently-deleted =
    Poistettu pysyvästi { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    }
items = Kohteita: { $items }
item-accessed = Käytetty: { $accessed }
type-to-search-enter-path = Kirjoittaa polun kansioon tai tiedostoon
eject = Poista asemasta
copy-to-title = Valitse mihin kopioidaan
move-to-title = Valitse mihin siirretään
pasted-image = Liitetty kuva
pasted-text = Liitetty teksti
pasted-video = Liitetty video
type-to-search-select = Valitsee ensimmäisen täsmäävän tiedoston tai kansion
deleting =
    Poistetaan { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } roskakorista ({ $progress })…
deleted =
    Poistettu { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } roskakorista
removing-from-recents =
    Poistetaan { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } viimeaikaisista
removed-from-recents =
    Poistettu { $items } { $items ->
        [one] kohde
       *[other] kohdetta
    } viimeaikaisista
