cosmic-files = COSMIC Filer
empty-folder = Mappen är tom
empty-folder-hidden = Mappen är tom (har dolda objekt)
no-results = Inga resultat hittades
filesystem = Filsystem
home = Hem
networks = Nätverk
notification-in-progress = Filåtgärder pågår
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
empty-trash-warning = Objekt i papperskorgen kommer att tas bort permanent

## Monteringsfel dialogruta

mount-error = Kan inte komma åt enheten

## Ny Fil/katalog dialogruta

create-new-file = Skapa ny fil
create-new-folder = Skapa ny mapp
file-name = Filnamn
folder-name = Mappnamn
file-already-exists = En fil med det namnet finns redan
folder-already-exists = En mapp med det namnet finns redan
name-hidden = Namn som börjar med "." kommer att vara dolda
name-invalid = Namnet får inte vara "{ $filename }"
name-no-slashes = Namnet får inte innehålla snedstreck

## Öppna/Spara dialogruta

cancel = Avbryt
create = Skapa
open = Öppna
open-file = Öppna fil
open-folder = Öppna mapp
open-in-new-tab = Öppna i en ny flik
open-in-new-window = Öppna i nytt fönster
open-item-location = Öppna objektets plats
open-multiple-files = Öppna flera filer
open-multiple-folders = Öppna flera mappar
save = Spara
save-file = Spara fil

## Öppna med dialogruta

open-with-title = Hur vill du öppna "{ $name }"?
browse-store = Bläddra i { $store }

## Byt namn dialogruta

rename-file = Byt namn på fil
rename-folder = Byt namn på mapp

## Ersätt dialogruta

replace = Byt ut
replace-title = "{ $filename }" finns redan på den här platsen
replace-warning = Vill du ersätta filen med den du sparar? Om du ersätter den kommer dess innehåll att skrivas över.
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
# Listvy
name = Namn
modified = Ändrad
trashed-on = Kastad
size = Storlek
# Framstegssidfot
details = Detaljer
dismiss = Avfärda meddelande
operations-running =
    { $running } { $running ->
        [one] åtgärd
       *[other] åtgärder
    } kör ({ $percent }%)...
operations-running-finished =
    { $running } { $running ->
        [one] åtgärd
       *[other] åtgärder
    } kör ({ $percent }%), { $finished } slutförda...
pause = Pausa
resume = Återuppta

# Kontextsidor


## Om


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
failed = Misslyckad
complete = Färdig
compressing =
    Komprimerar { $items } { $items ->
        [one] objekt
       *[other] objekt
    } från "{ $from }" till "{ $to }" ({ $progress })...
compressed =
    Komprimerade { $items } { $items ->
        [one] objekt
       *[other] objekt
    } från "{ $from }" till "{ $to }"
copy_noun = Kopiera
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
set-executable-and-launched = Gjorde "{ $name }" körbar och startade
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
renaming = Byter namn på "{ $from }" till "{ $to }"
renamed = Bytt namn på "{ $from }" till "{ $to }"
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
unknown-folder = okänd mapp

## Öppna med

menu-open-with = Öppna med...
default-app = { $name } (standard)

## Visa detaljer

show-details = Visa detaljer
type = Typ: { $mime }
items = Objekt: { $items }
item-size = Storlek: { $size }
item-created = Skapad: { $created }
item-modified = Ändrad: { $modified }
item-accessed = Åtkomst: { $accessed }
calculating = Beräknar...

## Egenskaper


## Inställningar

settings = Inställningar
single-click = Ett enkelklick för att öppna

### Utseende

appearance = Utseende
theme = Tema
match-desktop = Matcha skrivbordet
dark = Mörkt
light = Ljust

### Skriv för att söka

type-to-search = Skriv för att söka
type-to-search-recursive = Söker i den aktuella mappen och alla undermappar
type-to-search-enter-path = Anger sökvägen till mappen eller filen
# Kontext meny
add-to-sidebar = Lägg till i sidofält
compress = Komprimera
extract-here = Packa upp
new-file = Ny fil…
new-folder = Ny mapp…
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
list-directories-first = Lista mappar först
gallery-preview = Galleri förhandsvisning
menu-settings = Inställningar…
menu-about = Om COSMIC Filer...

## Sortera

sort = Sortera
sort-a-z = A-Ö
sort-z-a = Ö-A
sort-newest-first = Nyaste först
sort-oldest-first = Äldst först
sort-smallest-to-largest = Minsta till största
sort-largest-to-smallest = Största till minsta
remove = Ta bort
repository = Källkod
support = Support
grid-spacing = Rutnätsmellanrum
extract-password-required = Lösenord krävs
extract-to = Packa upp till...
extract-to-title = Packa upp till mapp
other-apps = Andra program
related-apps = Relaterade program
permanently-delete-question = Ta bort permanent?
delete = Ta bort
permanently-delete-warning = { $target } kommer att tas bort permanent. Detta kan inte göras ogjort.
none = Ingen
execute-only = Endast exekvera
write-only = Endast skriva
write-execute = Skriva och exekvera
read-only = Endast läsa
read-execute = Läsa och exekvera
read-write = Läsa och skriva
read-write-execute = Läsa, skriva och exekvera
favorite-path-error = Fel vid öppning av mapp
favorite-path-error-description =
    Kunde inte öppna "{ $path }"
    "{ $path }" finns kanske inte eller så har du inte behörighet att öppna den

    Vill du ta bort den från sidolisten?
keep = Behåll
progress-failed = { $percent }%, misslyckades
deleting =
    Raderar { $items } { $items ->
        [one] objekt
       *[other] objekt
    } från { trash } ({ $progress })...
deleted =
    Borttagna { $items } { $items ->
        [one] objekt
       *[other] objekt
    } från { trash }
setting-permissions = Sätter behörigheter för "{ $name }" till { $mode }
set-permissions = Satte behörigheter för "{ $name }" to { $mode }
permanently-deleting =
    Raderar { $items } { $items ->
        [one] objekt
       *[other] objekt
    } permanent
permanently-deleted =
    Permanent borttagna { $items } { $items ->
        [one] objekt
       *[other] objekt
    }
removing-from-recents =
    Tar bort { $items } { $items ->
        [one] objekt
       *[other] objekt
    } från { recents }
removed-from-recents =
    Tog bort { $items } { $items ->
        [one] objekt
       *[other] objekt
    } från { recents }
delete-permanently = Ta bort permanent
eject = Mata ut
remove-from-recents = Ta bort från senaste
reload-folder = Ladda om mapp
selected-items = De { $items } valda objekten
empty-trash-title = Töm papperskorgen?
type-to-search-select = Markerar den första matchande filen eller mappen
pasted-image = Inklistrad bild
pasted-text = Inklistrad text
pasted-video = Inklistrad video
