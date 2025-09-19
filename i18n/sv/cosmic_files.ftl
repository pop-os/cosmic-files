cosmic-files = COSMIC Files
empty-folder = Tom katalog
empty-folder-hidden = Tom katalog (har dolda objekt)
no-results = Inga resultat hittades
filesystem = Filsystem
home = Hem
networks = Nätverk
notification-in-progress = Filoperationer pågår.
trash = Papperskorg
recents = Senaste
undo = Ångra
today = Idag
# Skrivbordsvyalternativ
desktop-view-options = Skrivbordsvyalternativ...
show-on-desktop = Visa på skrivbord
desktop-folder-content = Skrivbordsmappinnehåll
mounted-drives = Monterade enheter
trash-folder-icon = Ikon för papperskorgen
icon-size-and-spacing = Ikonstorlek och mellanrum
icon-size = Ikonstorlek

# Dialogruta


# Dialogrutor


## Komprimera dialogruta

create-archive = Skapa arkiv

## Töm papperskorgen dialogruta

empty-trash = Töm papperskorgen
empty-trash-warning = Är du säker på att du vill ta bort alla objekt i papperskorgen permanent?

## Monteringsfel dialogruta

mount-error = Kan inte komma åt enheten

## Ny Fil/katalog dialogruta

create-new-file = Skapa ny fil
create-new-folder = Skapa ny katalog
file-name = Filnamn
folder-name = Katalognamn
file-already-exists = En fil med det namnet finns redan.
folder-already-exists = En katalog med det namnet finns redan.
name-hidden = Namn som börjar med "." kommer att vara dolda.
name-invalid = Namnet kan inte vara "{ $filename }".
name-no-slashes = Namnet får inte innehålla snedstreck.

## Öppna/Spara dialogruta

cancel = Avbryt
create = Skapa
open = Öppna
open-file = Öppna fil
open-folder = Öppna katalog
open-in-new-tab = Öppna i en ny flik
open-in-new-window = Öppna i nytt fönster
open-item-location = Öppna objektets plats
open-multiple-files = Öppna flera filer
open-multiple-folders = Öppna flera kataloger
save = Spara
save-file = Spara fil

## Öppna med dialogruta

open-with-title = Hur vill du öppna "{ $name }"?
browse-store = Bläddra i { $store }

## Byt namn dialogruta

rename-file = Byt namn på fil
rename-folder = Byt namn på katalog

## Ersätt dialogruta

replace = Ersätt
replace-title = "{ $filename }" existerar redan på den här platsen.
replace-warning = Vill du ersätta den med den du sparar? Om du ersätter den kommer dess innehåll att skrivas över.
replace-warning-operation = Vill du ersätta den? Om du ersätter den kommer dess innehåll att skrivas över.
original-file = Originalfil
replace-with = Ersätt med
apply-to-all = Verkställ för alla
keep-both = Behåll båda
skip = Hoppa över

## Ställ in som körbar och starta dialogruta

set-executable-and-launch = Gör körbar och starta
set-executable-and-launch-description = Vill du göra "{ $name }" körbar och starta den?
set-and-launch = Ställ in och starta

## Metadata dialogruta

open-with = Öppna med
owner = Ägare
group = Grupp
other = Andra
read = Läs
write = Skriv
execute = Exekvera
# Listvy
name = Namn
modified = Modifierad
trashed-on = Kastad
size = Storlek
# Framstegssidfot
details = Detaljer
dismiss = Stäng meddelande
operations-running = { $running } operationer körs ({ $percent }%)...
operations-running-finished = { $running } operationer körs ({ $percent }%), { $finished } färdig...
pause = Paus
resume = Återuppta

# Kontextsidor


## Om

git-description = Git commit { $hash } på { $date }

## Lägg till en Nätverksenhet

add-network-drive = Lägg till en Nätverksenhet
connect = Anslut
connect-anonymously = Anslut anonymt
connecting = Ansluter...
domain = Domän
enter-server-address = Ange serveradress
try-again = Försök igen
username = Användarnamn
network-drive-description =
       Serveradresser består av ett protokollprefix och en adress.
    Exempel: ssh://192.168.0.1, ftp://[2001:db8::1]

### Se till att behålla kommatecken som skiljer kolumnerna åt

network-drive-schemes =
     Tillgängliga protokoll, Prefix
     AppleTalk,afp://
     File Transfer Protocol,ftp:// eller ftps://
     Network File System (NFS),nfs://
     Server Message Block (SMB),smb://
    SSH-filöverföringsprotokoll,sftp:// eller ssh://
     WebDav,dav:// eller davs://
network-drive-error = Kan inte komma åt nätverksenheten
password = Lösenord
remember-password = Kom ihåg lösenord

## Operationer

cancelled = Avbruten
edit-history = Redigera historik
history = Historik
no-history = Inga objekt i historiken.
pending = Väntar
progress = { $percent }%
progress-cancelled = { $percent }%, avbruten
progress-paused = { $percent }%, pausad
failed = Misslyckades
complete = Färdig
compressing =
    Komprimerar { $items } { $items ->
        [one] item
       *[other] items
    } from "{ $from }" to "{ $to }" ({ $progress })...
compressed =
    Komprimerade { $items } { $items ->
        [one] item
       *[other] items
    } from "{ $from }" to "{ $to }"
copy_noun = Koperia
creating = Skapar "{ $name }" i "{ $parent }"
created = Skapade "{ $name }" i "{ $parent }"
copying =
    Kopierar { $items } { $items ->
        [one] objekt
       *[other] flera objekt
    } från "{ $from }" till "{ $to }" ({ $progress })...
copied =
    Kopierade { $items } { $items ->
        [one] objekt
       *[other] flera objekt
    } från "{ $from }" till "{ $to }"
emptying-trash = Tömmer { trash } ({ $progress })...
emptied-trash = Tömde { trash }
extracting =
    Packar upp { $items } { $items ->
        [one] objekt
       *[other] flera objekt
    } från "{ $from }" till "{ $to }" ({ $progress })...
extracted =
    Packade upp { $items } { $items ->
        [one] objekt
       *[other] flera objekt
    } från "{ $from }" till "{ $to }"
setting-executable-and-launching = Gör "{ $name }" körbar och startar
set-executable-and-launched = Gör "{ $name }" körbar och startar
moving =
    Flyttar { $items } { $items ->
        [one] objekt
       *[other] flera objekt
    } från "{ $from }" till "{ $to }" ({ $progress })...
moved =
    Flyttade { $items } { $items ->
        [one] objekt
       *[other] flera objekt
    } från "{ $from }" till "{ $to }"
renaming = Byter namn "{ $from }" till "{ $to }"
renamed = Bytt namn "{ $from }" till "{ $to }"
restoring =
    Återställer { $items } { $items ->
        [one] objekt
       *[other] flera objekt
    } från { trash } ({ $progress })...
restored =
    Återställt { $items } { $items ->
        [one] objekt
       *[other] flera objekt
    } från { trash }
unknown-folder = okänd katalog

## Öppna med

menu-open-with = Öppna med...
default-app = { $name } (default)

## Visa detaljer

show-details = Visa detaljer
type = Typ: { $mime }
items = Objekt: { $items }
item-size = Storlek: { $size }
item-created = Skapad: { $created }
item-modified = Modifierad: { $modified }
item-accessed = Åtkomst: { $accessed }
calculating = Beräknar...

## Egenskaper

properties = Egenskaper

## Inställningar

settings = Inställningar
single-click = Ett enkelklick för att öppna

### Utseende

appearance = Utseende
theme = Tema
match-desktop = Matcha skrivbordet
dark = Mörk
light = Ljus

### Skriv för att söka

type-to-search = Skriv för att söka
type-to-search-recursive = Söker i den aktuella mappen och alla undermappar
type-to-search-enter-path = Anger sökvägen till katalogen eller filen
# Kontext meny
add-to-sidebar = Lägg till i sidofält
compress = Komprimera
extract-here = Packa upp
new-file = Ny fil
new-folder = Ny katalog
open-in-terminal = Öppna i terminal
move-to-trash = Flytta till papperskorg
restore-from-trash = Återställ från papperskorgen
remove-from-sidebar = Ta bort från sidofält
sort-by-name = Sortera efter namn
sort-by-modified = Sortera efter senast ändrad
sort-by-size = Sortera efter storlek
sort-by-trashed = Sortera efter borttagningstid

## Skrivbord

change-wallpaper = Byt bakgrund...
desktop-appearance = Skrivbordsutseende...
display-settings = Skärminställningar...

# Meny


## Fil

file = Fil
new-tab = Ny flik
new-window = Nytt fönster
rename = Byt namn...
menu-show-details = Visa detaljer...
close-tab = Stäng flik
quit = Avsluta

## Redigera

edit = Redigera
cut = Klipp ut
copy = Kopiera
paste = Klistra in
select-all = Välj alla

## Visa

zoom-in = Zooma in
default-size = Standardstorlek
zoom-out = Zooma ut
view = Visa
grid-view = Rutnätsvy
list-view = Listvy
show-hidden-files = Visa dolda filer
list-directories-first = Lista kataloger först
gallery-preview = Galleri förhandsvisning
menu-settings = Inställningar...
menu-about = Om COSMIC Files...

## Sortera

sort = Sortera
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Nyaste först
sort-oldest-first = Äldst först
sort-smallest-to-largest = Minsta till största
sort-largest-to-smallest = Största till minsta
remove = Ta bort
