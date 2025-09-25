cosmic-files = Soubory COSMIC
empty-folder = Složka je prázdná
empty-folder-hidden = Složka je prázdná (obsahuje skryté položky)
filesystem = Souborový systém
home = Domů
recents = Nedávné
trash = Koš
# List view
name = Název
modified = Datum změny
size = Velikost

# Dialogs


## Empty Trash Dialog

empty-trash = Vysypat koš
empty-trash-warning = Opravdu chcete smazat všechny položky z koše?

## New File/Folder Dialog

create-new-file = Vytvořit nový soubor
create-new-folder = Vytvořit novou složku
file-name = Název souboru
folder-name = Název složky
file-already-exists = Soubor s daným názvem již existuje.
folder-already-exists = Složka s daným názvem již existuje.
name-hidden = Položky s názvem začínajícím na "." budou skryty.
name-invalid = Název nemůže být "{ $filename }".
name-no-slashes = Název nesmí obsahovat lomítka.

## Open/Save Dialog

cancel = Zrušit
open = Otevřít
open-file = Otevřít soubor
open-folder = Otevřít složku
open-in-new-tab = Otevřít na nové kartě
open-in-new-window = Ovevřít v novém okně
open-multiple-files = Otevřít více souborů
open-multiple-folders = Otevřít více složek
save = Uložit
save-file = Uložit soubor

## Rename Dialog

rename-file = Přejmenovat soubor
rename-folder = Přejmenovat složku

## Replace Dialog

replace = Nahradit
replace-title = "{ $filename }" již na daném místě existuje.
replace-warning = Chcete nahradit soubor tím, který ukládáte? Nahrazení přepíše veškerý jeho obsah.

# Context Pages


## About

git-description = Git commit { $hash } dne { $date }

## Operations

operations = Operace
pending = Čekající
failed = Selhalo
complete = Dokončené
copy_noun = Kopírovat

## Open with

menu-open-with = Otevřít pomocí...
default-app = { $name } (výchozí)

## Properties

properties = Vlastnosti

## Settings

settings = Nastavení

### Appearance

appearance = Vzhled
theme = Téma
match-desktop = Dle systému
dark = Tmavé
light = Světlý
# Context menu
add-to-sidebar = Přidat do postranního panelu
new-file = Nový soubor...
new-folder = Nová složka...
open-in-terminal = Otevřít v terminálu
move-to-trash = Přesunout do koše
restore-from-trash = Obnovit z koše
remove-from-sidebar = Odstranit z postranního panelu
sort-by-name = Seřadit podle názvu
sort-by-modified = Seřadit podle data změny
sort-by-size = Seřadit podle velikosti

# Menu


## File

file = Soubor
new-tab = Nová karta
new-window = Nové okno
rename = Přejmenovat...
close-tab = Zavřít kartu
quit = Ukončit

## Edit

edit = Upravit
cut = Vyjmout
copy = Kopírovat
paste = Vložit
select-all = Vybrat vše

## View

zoom-in = Přiblížit
default-size = Výchozí velikost
zoom-out = Oddálit
view = Zobrazit
grid-view = Zobrazit jako mřížku
list-view = Zobrazit jako seznam
show-hidden-files = Zobrazit skryté soubory
list-directories-first = Řadit nejprve složky
menu-settings = Nastavení...
menu-about = O aplikaci Soubory COSMIC...
no-results = Nenalezeny žádné výsledky
repository = Repozitář
support = Podpora
networks = Sítě
notification-in-progress = Probíhají operace se soubory.
undo = Vrátit
connect = Připojit
today = Dnes
desktop-view-options = Možnosti zobrazení plochy...
show-on-desktop = Zobrazit na ploše
desktop-folder-content = Obsah složky na ploše
mounted-drives = Připojené disky
trash-folder-icon = Ikona koše
icon-size = Velikost ikony
password = Heslo
remove = Odstranit
username = Uživatelské jméno
details = Detaily
pause = Pozastavit
resume = Pokračovat
create-archive = Vytvořit archív
extract-password-required = Vyžadováno heslo
extract-to = Rozbalit do...
extract-to-title = Rozbalit do složky
mount-error = Nelze přistoupit k disku
create = Tvorba
open-item-location = Otevřít umístění položky
open-with-title = Jak chcete otevřít "{ $name }"?
browse-store = Procházet { $store }
other-apps = Ostatní aplikace
related-apps = Související aplikace
permanently-delete-question = Trvale smazat
delete = Smazat
permanently-delete-warning = Opravdu chcete trvale smazat { $target }? Tuto operaci nelze vrátit.
replace-warning-operation = Chcete soubor nahradit? Nahrazení přepíše veškerý jeho obsah.
original-file = Původní soubor
replace-with = Nahradit za
keep-both = Ponechat oba
skip = Přeskočit
set-executable-and-launch = Umožnit spouštění a spustit
set-executable-and-launch-description = Chcete umožnit spouštění pro "{ $name }" a následně ho spustit?
set-and-launch = Nastavit a spustit
open-with = Otevřít pomocí
other = Ostatní
none = Žádný
icon-size-and-spacing = Velikost a rozestupy ikon
grid-spacing = Rozestupy mřížky
deleting =
    Mazání { $items } { $items ->
        [one] položky
       *[other] položek
    } z { trash } ({ $progress })...
sort-by-trashed = Seřadit podle času smazání
deleted =
    { $items } { $items ->
        [one] položka smazána
        [few] položky smazány
        [many] položek smazáno
       *[other] položek smazáno
    } z { trash }
emptying-trash = Vysypávám { trash } ({ $progress })...
emptied-trash = { trash } byl vysypán
restoring =
    Obnovování { $items } { $items ->
        [one] položky
       *[other] položek
    } z { trash } ({ $progress })...
restored =
    { $items } { $items ->
        [one] položka obnovena
        [few] položky obnoveny
        [many] položek obnoveno
       *[other] položek obnoveno
    } z { trash }
permanently-deleted =
    { $items } { $items ->
        [one] položka trvale smazána
        [few] položky trvale smazány
        [many] položek trvale smazáno
       *[other] položek trvale smazáno
    }
delete-permanently = Smazat trvale
trashed-on = Smazáno
dismiss = Zavřít zprávu
operations-running =
    Běží { $running } { $running ->
        [one] operace
        [few] operace
        [many] operací
       *[other] operací
    } ({ $percent }%)...
operations-running-finished =
    Běží { $running } { $running ->
        [one] operace
        [few] operace
        [many] operací
       *[other] operací
    } ({ $percent }%), { $finished } dokončeno...
apply-to-all = Použít na vše
owner = Vlastník
group = Skupina
execute-only = Pouze spouštění
write-only = Pouze zápis
write-execute = Zápis a spouštění
read-only = Pouze čtení
add-network-drive = Přidat síťový disk
connect-anonymously = Připojit se anonymně
connecting = Připojování...
domain = Doména
enter-server-address = Zadejte adresu serveru
network-drive-description =
    Adresy serveru obsahují prefix protokolu a adresu.
    Příklady: ssh://192.168.0.1, ftp://[2001:db8::1]
network-drive-schemes =
    Dostupné protokoly,Prefix
    AppleTalk,afp://
    File Transfer Protocol,ftp:// nebo ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// nebo ssh://
    WebDAV,dav:// nebo davs://
network-drive-error = Nelze přistoupit k síťovému disku
remember-password = Zapamatovat heslo
try-again = Zkuste to znovu
cancelled = Zrušeno
edit-history = Upravit historii
history = Historie
no-history = Žádné položky v historii.
progress = { $percent }%
progress-cancelled = { $percent }%, zrušeno
progress-failed = { $percent }%, selhalo
progress-paused = { $percent }%, pozastaveno
keep = Ponechat
compressing =
    Komprimování { $items } { $items ->
        [one] položky
       *[other] položek
    } z "{ $from }" do "{ $to }" ({ $progress })...
compressed =
    { $items } { $items ->
        [one] položka zkomprimována
        [few] položky zkomprimovány
        [many] položek zkomprimováno
       *[other] položek zkomprimováno
    } z "{ $from }" do "{ $to }"
creating = Vytváření "{ $name }" v "{ $parent }"
created = Vytvořen "{ $name }" v "{ $parent }"
copying =
    Kopírování { $items } { $items ->
        [one] položky
       *[other] položek
    } z "{ $from }" do "{ $to }" ({ $progress })...
copied =
    { $items } { $items ->
        [one] položka zkopírována
        [few] položky zkopírovány
        [many] položek zkopírováno
       *[other] položek zkopírováno
    } z "{ $from }" do "{ $to }"
extracting =
    Extrahování { $items } { $items ->
        [one] položky
       *[other] položek
    } z "{ $from }" do "{ $to }" ({ $progress })...
favorite-path-error-description =
    Nelze otevřít "{ $path }".
    Cesta buď neexistuje nebo nemáte dostatečná práva pro otevření.

    Chcete položku odstranit z bočního panelu?
selected-items = { $items } vybraných položek
read-execute = Čtení a spouštění
read-write-execute = Čtení, zápis a spouštění
read-write = Čtení a zápis
favorite-path-error = Chyba otevírání složky
extracted =
    { $items } { $items ->
        [one] položka extrahována
        [few] položky extrahovány
        [many] položek extrahováno
       *[other] položek extrahováno
    } z "{ $from }" do "{ $to }"
setting-executable-and-launching = Nastavování "{ $name }" jako spustitelného souboru a spouštění
set-executable-and-launched = "{ $name }" nastaven jako spustitelný soubor a spuštěn
setting-permissions = Nastavování práv pro "{ $name }" na { $mode }
set-permissions = Práva pro "{ $name }" nastavena na { $mode }
moving =
    Přesouvání { $items } { $items ->
        [one] položky
       *[other] položek
    } z "{ $from }" do "{ $to }" ({ $progress })...
moved =
    { $items } { $items ->
        [one] položka přesunuta
        [few] položky přesunuty
        [many] položek přesunuto
       *[other] položek přesunuto
    } z "{ $from }" do "{ $to }"
permanently-deleting =
    Trvalé mazání { $items } { $items ->
        [one] položky
       *[other] položek
    }
removing-from-recents =
    Odstraňování { $items } { $items ->
        [one] položky
       *[other] položek
    } z { recents }
removed-from-recents =
    { $items } { $items ->
        [one] položka odstraněna
        [few] položky odstraněny
        [many] položek odstraněno
       *[other] položek odstraněno
    } z { recents }
remove-from-recents = Odstranit z nedávných
renaming = Přejmenování "{ $from }" na "{ $to }"
renamed = Přejmenováno "{ $from }" na "{ $to }"
unknown-folder = neznámá složka
show-details = Zobrazit detaily
type = Typ: { $mime }
items = Položky: { $items }
item-size = Velikost: { $size }
item-created = Vytvořeno: { $created }
item-modified = Změněno: { $modified }
item-accessed = Přístup: { $accessed }
calculating = Vypočítávání...
single-click = Otevřít jedním kliknutím
type-to-search = Vyhledávání psaním
type-to-search-recursive = Prohledává současnou složku a její podsložky
type-to-search-enter-path = Zadává cestu ke složce nebo souboru
compress = Komprimovat
eject = Vysunout
extract-here = Extrahovat
change-wallpaper = Změnit tapetu...
desktop-appearance = Vzhled plochy...
display-settings = Nastavení displeje...
reload-folder = Znovu načíst složku
sort-z-a = Z-A
sort-newest-first = Nejnovější první
sort-oldest-first = Nejstarší první
sort-smallest-to-largest = Od nejmenšího po největší
sort-largest-to-smallest = Od největšího po nejmenší
gallery-preview = Náhled galerie
sort = Seřadit
sort-a-z = A-Z
