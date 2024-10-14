cosmic-files = Súbory COSMIC
empty-folder = Prázdny priečinok
empty-folder-hidden = Prázdny priečinok (obsahuje skryté položky)
no-results = Žiadne výsledky
filesystem = Súborový systém
home = Domov
networks = Sieť
notification-in-progress = Prebiehajú operácie so súbormi
trash = Kôš
recents = Nedávne položky
undo = Späť
today = Dnes

# Desktop view options
desktop-view-options = Nastavenia zobrazenia na ploche...
show-on-desktop = Zobraziť na ploche
desktop-folder-content = Obsah priečinku plocha
mounted-drives = Pripojené disky
trash-folder-icon = Ikona koša
icon-size-and-spacing = Veľkosť a rozostup medzi ikonami
icon-size = Veľkosť ikon

# List view
name = Meno
modified = Dátum úpravy
trashed-on = Odstránené
size = Veľkosť

# Dialogs

## Compress Dialog
create-archive = Vytvoriť archív

## Empty Trash Dialog
empty-trash = Vysypať kôš
empty-trash-warning = Ste si istý, že chcete trvalo vymazať všetky položky v koši?

## Mount Error Dialog
mount-error = Nepodarilo sa pristúpiť ku zariadeniu

## New File/Folder Dialog
create-new-file = Vytvoriť nový súbor
create-new-folder = Vytvoriť nový priečinok
file-name = Názov súboru
folder-name = Názov priečinku
file-already-exists = Súbor s rovnakým názvom už existuje.
folder-already-exists = Priečinok s rovnakým názvom už existuje.
name-hidden = Názvy začínajúce na "." budú skryté.
name-invalid = Názov nesmie byť "{$filename}".
name-no-slashes = Názov nesmie obsahovať lomítko.

## Open/Save Dialog
cancel = Zrušiť
create = Vytvoriť
open = Otvoriť
open-file = Otvoriť súbor
open-folder = Otvoriť priečinok
open-in-new-tab = Otvoriť na novej karte
open-in-new-window = Otvoriť v novom okne
open-item-location = Otvoriť umiestenie položky
open-multiple-files = Otvoriť viacero súborov
open-multiple-folders = Otvoriť viacero priečinkov
save = Uložiť
save-file = Uložiť súbor

## Open With Dialog
open-with-title = Ako chcete otvoriť "{$name}"?
browse-store = Prehľadať {$store}

## Rename Dialog
rename-file = Premenovať súbor
rename-folder = Premenovať priečinok

## Replace Dialog
replace = Nahradiť
replace-title = {$filename} už existuje v tomto umiestnení.
replace-warning = Chcete ho nahradiť tým, ktorý práve ukladáte? Nahradením bude prepísaný jeho obsah.
replace-warning-operation = Chcete ho nahradiť? Nahradením bude jeho obsah prepísaný.
original-file = Pôvodný súbor
replace-with = Nahradiť s
apply-to-all = Použiť na všetky
keep-both = Uchovať oba
skip = Preskočiť

## Set as Executable and Launch Dialog
set-executable-and-launch = Nastaviť ako spustiteľné a spustiť
set-executable-and-launch-description = Chcete nastaviť "{$name}" ako spustiteľné a spustiť?
set-and-launch = Nastaviť a spustiť

## Metadata Dialog
owner = Majiteľ
group = Skupina
other = Iné
read = Čítanie
write = Zapisovanie
execute = Spustenie

# Context Pages

## About
git-description = Git commit {$hash} o {$date}

## Add Network Drive
add-network-drive = Pridať sieťový disk
connect = Pripojiť
connect-anonymously = Pripojiť sa anonymne
connecting = Pripája sa...
domain = Doména
enter-server-address = Zadajte názov servera
network-drive-description =
    Adresa servera obsahuje protokol a adresu.
    Ukážka: ssh://192.168.0.1, ftp://[2001:db8::1]
### Make sure to keep the comma which separates the columns
network-drive-schemes =
    Dostupné protokoly,Prefix
    AppleTalk,afp://
    File Transfer Protocol,ftp:// alebo ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// alebo ssh://
    WebDav,dav:// alebo davs://
network-drive-error = Nepodarilo sa pripojiť ku sieťovému disku
password = Heslo
remember-password = Zapamätať heslo
try-again = Skúsiť znovu
username = Prihlasovanie meno

## Operations
edit-history = História zmien
history = História
no-history = Žiadne položky v histórií.
pending = V poradí
failed = Zlyhalo
complete = Dokončené
compressing = Komprimujem {$items} {$items ->
        [one] položku
        [few] položky
        *[other] položiek
    } z "{$from}" do "{$to}"
compressed = {$items} {$items ->
        [one] položka zkomprimovaná
        [few] položky zkomprimované
        *[other] položiek zkomprimovaných
    } z "{$from}" do "{$to}"
copy_noun = Skopírované
creating = Vytváram {$name} v {$parent}
created = Vytvorené {$name} v {$parent}
copying = Kopírujem {$items} {$items ->
        [one] položku
        [few] položky
        *[other] položiek
    } z {$from} do {$to}
copied = {$items ->
        [one] Skopírovaná {$items} položka
        [few] Skopírované {$items} položky
        *[other] Skopírovaných {$items} položiek
    } z {$from} do {$to}
emptying-trash = Kôš sa vysypáva
emptied-trash = Kôš bol vysypaný
extracting = Extrahujem {$items} {$items ->
        [one] položku
        [few] položky
        *[other] položiek
    } z "{$from}" do "{$to}"
extracted = {$items} {$items ->
        [one] položka extrahovaná
        [few] položky extrahované
        *[other] položiek extrahovaných
    } z "{$from}" do "{$to}"
setting-executable-and-launching = Nastavuje "{$name}" ako spustiteľné a spúšťam
set-executable-and-launched = "{$name}" bolo nastavené ako spustiteľné a bolo spustené
moving = Presúvam {$items} {$items ->
        [one] položku
        [few] položky
        *[other] položiek
    } z {$from} do {$to}
moved = {$items ->
        [one] Presunutá {$items} položka
        [few] Presunuté {$items} položky
        *[other] Presunutých {$items} položiek
    } z {$from} do {$to}
renaming = Premenovávanie z {$from} na {$to}
renamed = Premenované z {$from} na {$to}
restoring = Obnovujem {$items} {$items ->
        [one] položku
        [few] položky
        *[other] položiek
    } z koša 
restored = {$items ->
        [one] Obnovená {$items} položka
        [few] Obnovené {$items} položky
        *[other] Obnovených {$items} položiek
    } z koša 
undo = Späť
unknown-folder = neznámy priečinok

## Open with
open-with = Otvoriť s
default-app = {$name} (Predvolené)

## Show details
show-details = Podrobnosti

## Settings
settings = Nastavenia

### Appearance
appearance = Vzhľad
theme = Téma
match-desktop = Podľa systému
dark = Tmavá
light = Svetlá

# Context menu
add-to-sidebar = Pridať do panelu
compress = Komprimovať
extract-here = Extrahovať
new-file = Nový súbor
new-folder = Nový priečinok
open-in-terminal = Otvoriť v termináli
move-to-trash = Presunúť do koša
restore-from-trash = Obnoviť z koša
remove-from-sidebar = Odstrániť z panelu
sort-by-name = Zoradiť podľa mena
sort-by-modified = Zoradiť podľa dátumu úpravy
sort-by-size = Zoradiť podľa veľkosti
sort-by-trashed = Zoradiť podľa dátumu odstránenia

## Desktop
change-wallpaper = Zmeniť pozadie...
desktop-appearance = Nastavenia vzhľadu...
display-settings = Nastavenia obrazovky...

# Menu

## File
file = Súbor
new-tab = Nová karta
new-window = Nové okno
rename = Premenovať
close-tab = Zatvoriť kartu
quit = Ukončiť

## Edit
edit = Upraviť
cut = Vystrihnúť
copy = Kopírovať
paste = Prilepiť
select-all = Vybrať všetko

## View
zoom-in = Priblížiť
default-size = Predvolená veľkosť
zoom-out = Oddialiť
view = Zobraziť
grid-view = Zobrazenie mriežky
list-view = Zobrazenie zoznamu
show-hidden-files = Zobraziť skryté súbory
list-directories-first = Zobraziť priečinky ako prvé
gallery-preview = Rýchla ukážka
menu-settings = Nastavenia...
menu-about = O aplikácií...


## Sort
sort = Zoradiť
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Najnovšie ako prvé
sort-oldest-first = Najstaršie ako prvé
sort-smallest-to-largest = Vzostupne
sort-largest-to-smallest = Zostupne
