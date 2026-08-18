cosmic-files = COSMIC Filhåndtering
empty-folder = Tom mappe
empty-folder-hidden = Tom mappe (har skjulte emner)
no-results = Fandt ingen resultater
filesystem = Filsystem
home = Hjem
networks = Netværk
notification-in-progress = Fil-operationer er igang.
trash = Papirkurv
recents = Seneste
undo = Fortryd
today = I dag
# Desktop view options
desktop-view-options = Valgmuligheder for skrivebordsvisning...
show-on-desktop = Vis på Skrivebord
desktop-folder-content = Skrivebords-mappeindhold
mounted-drives = Monterede drev
trash-folder-icon = Papirkurv-ikon
icon-size-and-spacing = Ikonstørrelse og -afstand
icon-size = Ikonstørrelse
# List view
name = Navn
modified = Ændret
trashed-on = Kasseret
size = Størrelse
# Progress footer
details = Detaljer
dismiss = Luk meddelelse
operations-running =
    { $running } { $running ->
        [one] operation
       *[other] operationer
    } kører ({ $percent }%)...
operations-running-finished =
    { $running } { $running ->
        [one] operation
       *[other] operationer
    } kører ({ $percent }%), { $finished } færdig...
pause = Pause
resume = Genoptag

# Dialogs


## Compress Dialog

create-archive = Opret arkiv

## Empty Trash Dialog

empty-trash = Tøm papirkurv
empty-trash-warning = Emner i Papirkurv-mappen vil blive slettet permanent

## Mount Error Dialog

mount-error = Ude af stand til at tilgå drev

## New File/Folder Dialog

create-new-file = Opret ny fil
create-new-folder = Opret ny mappe
file-name = Filnavn
folder-name = Mappenavn
file-already-exists = En fil med det navn eksisterer allerede
folder-already-exists = En mappe med det navn eksisterer allerede
name-hidden = Navne begyndende med "." vil være skjulte
name-invalid = Navn må ikke være "{ $filename }"
name-no-slashes = Navn må ikke indeholde skråstreger

## Open/Save Dialog

cancel = Annuller
create = Opret
open = Åbn
open-file = Åbn fil
open-folder = Åbn mappe
open-in-new-tab = Åbn i nyt faneblad
open-in-new-window = Åbn i nyt vindue
open-item-location = Åbn emne-placering
open-multiple-files = Åbn flere filer
open-multiple-folders = Åbn flere mapper
save = Gem
save-file = Gem fil

## Open With Dialog

open-with-title = Hvordan vil du åbne "{ $name }"?
browse-store = Gennemse { $store }

## Rename Dialog

rename-file = Omdøb fil
rename-folder = Omdøb mappe

## Replace Dialog

replace = Erstat
replace-title = "{ $filename }" eksisterer allerede i denne placering.
replace-warning = Ønsker du at erstatte den med den, du er ved at gemme? Erstatning af den vil overskrive dens indhold.
replace-warning-operation = Ønsker du at erstatte den? Erstatning af den vil overskrive dens indhold.
original-file = Original fil
replace-with = Erstat med
apply-to-all = Anvend på alle
keep-both = Behold begge
skip = Spring over

## Set as Executable and Launch Dialog

set-executable-and-launch = Angiv som eksekverbar og start
set-executable-and-launch-description = Vil du angive "{ $name }" som eksekverbar og starte den?
set-and-launch = Angiv og start

## Metadata Dialog

owner = Ejer
group = Gruppe
other = Andet

# Context Pages


## About


## Add Network Drive

add-network-drive = Tilføj netværksdrev
connect = Opret forbindelse
connect-anonymously = Opret forbindelse anonymt
connecting = Opretter forbindelse...
domain = Domæne
enter-server-address = Indtast serveradresse
network-drive-description =
    Server-adresser inkluderer et protokolpræfiks og adresse.
    Eksempler: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Tilgængelige protokoller,Præfiks
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = Kan ikke tilgå netværksdrev
password = Adgangskode
remember-password = Husk adgangskode
try-again = Forsøg igen
username = Brugernavn

## Operations

cancelled = Annulleret
edit-history = Redigér historie
history = Historie
no-history = Ingen historik.
pending = Afventer
progress = { $percent }%
progress-cancelled = { $percent }%, annulleret
progress-paused = { $percent }%, sat på pause
failed = Mislykkedes
complete = Færdigt
compressing =
    Komprimerer { $items } { $items ->
        [one] objekt
       *[other] objekter
    } fra "{ $from }" til "{ $to }" ({ $progress })...
compressed =
    Komprimeret { $items } { $items ->
        [one] objekt
       *[other] objekter
    } fra "{ $from }" til "{ $to }"
copy_noun = Kopi
creating = Opretter "{ $name }" i "{ $parent }"
created = Oprettet "{ $name }" i "{ $parent }"
copying =
    Kopierer { $items } { $items ->
        [one] objekt
       *[other] objekter
    } fra "{ $from }" til "{ $to }" ({ $progress })...
copied =
    Kopieret { $items } { $items ->
        [one] objekt
       *[other] objekter
    } fra "{ $from }" til "{ $to }"
emptying-trash = Tømmer { trash } ({ $progress })...
emptied-trash = Tømt { trash }
extracting =
    Udpakker { $items } { $items ->
        [one] objekt
       *[other] objekter
    } fra "{ $from }" til "{ $to }" ({ $progress })...
extracted =
    Udpakket { $items } { $items ->
        [one] objekt
       *[other] objekter
    } fra "{ $from }" til "{ $to }"
setting-executable-and-launching = Indstiller "{ $name }" som eksekverbar fil og starter
set-executable-and-launched = Indstillet "{ $name }" som eksekverbar fil og startet
moving =
    Flytter { $items } { $items ->
        [one] objekt
       *[other] objekter
    } fra "{ $from }" til "{ $to }" ({ $progress })...
moved =
    Flyttet { $items } { $items ->
        [one] objekt
       *[other] objekter
    } fra "{ $from }" til "{ $to }"
renaming = Omdøber "{ $from }" til "{ $to }"
renamed = Omdøbt "{ $from }" til "{ $to }"
restoring =
    Genopretter { $items } { $items ->
        [one] objekt
       *[other] objekter
    } fra { trash } ({ $progress })...
restored =
    Genoprettet { $items } { $items ->
        [one] objekt
       *[other] objekter
    } fra { trash }
unknown-folder = ukendt mappe

## Open with

menu-open-with = Åbn med...
default-app = { $name } (standardindstilling)

## Show details

show-details = Vis detaljer
type = Type: { $mime }
items = Objekter: { $items }
item-size = Størrelse: { $size }
item-created = Oprettet: { $created }
item-modified = Ændret: { $modified }
item-accessed = Tilgået: { $accessed }
calculating = Beregner...

## Settings

settings = Indstillinger

### Appearance

appearance = Udseende
theme = Tema
match-desktop = Følg skrivebordets indstillinger
dark = Mørkt
light = Lyst
# Context menu
add-to-sidebar = Tilføj til sidebjælke
compress = Komprimer…
extract-here = Extract
new-file = Ny fil...
new-folder = Ny mappe...
open-in-terminal = Åbn i terminal
move-to-trash = Flyt til skraldespand
restore-from-trash = Genopret fra skraldespand
remove-from-sidebar = Fjern fra sidebjælke
sort-by-name = Sortér efter navn
sort-by-modified = Sortér efter ændret
sort-by-size = Sortér efter størrelse
sort-by-trashed = Sortér efter sletningsdato

## Desktop

change-wallpaper = Skift baggrundsbillede...
desktop-appearance = Skrivebordsudseende...
display-settings = Skærmindstillinger...

# Menu


## File

file = Fil
new-tab = Ny fane
new-window = Nyt vindue
rename = Omdøb...
close-tab = Luk fane
quit = Afslut

## Edit

edit = Redigér
cut = Klip
copy = Kopiér
paste = Indsæt
select-all = Vælg alle

## View

zoom-in = Zoom ind
default-size = Standardstørrelse
zoom-out = Zoom ud
view = Vis
grid-view = Gitter-visning
list-view = Liste-visning
show-hidden-files = Vis skjulte filer
list-directories-first = List mapper først
gallery-preview = Galleri-forhåndsvisning
menu-settings = Indstillinger...
menu-about = Om COSMIC Filer...

## Sort

sort = Sortér
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Nyeste først
sort-oldest-first = Ældste først
sort-smallest-to-largest = Mindste til største
sort-largest-to-smallest = Største til mindste
delete = Slet
run = Kør
repository = Pakkearkiv
grid-spacing = Gitter-afstand
extract-password-required = Adgangskode påkrævet
extract-to = Udpak Til...
extract-to-title = Udpak til mappe
empty-trash-title = Tøm papirkurv?
keywords = Mappe;Håndtering;
comment = Filhåndteringsprogram til COSMIC-skrivebordet
copy-to-title = Vælg kopi destination
copy-to-button-label = Kopier
move-to-title = Vælg flytte destination
move-to-button-label = Flyt
deleted =
    Slettede { $items } { $items ->
        [one] element
       *[other] elementer
    } fra papirkurven
removing-from-recents =
    Fjerner { $items } { $items ->
        [one] element
       *[other] elementer
    } fra { recents }
type-to-search-enter-path = Indtast stien til mappen eller filen
permanently-deleting =
    Sletter permanent { $items } { $items ->
        [one] element
       *[other] elementer
    }
execute-only = Kun kørsel
removed-from-recents =
    Fjernede { $items } { $items ->
        [one] element
       *[other] elementer
    } fra { recents }
permanently-deleted =
    Slettede permanent { $items } { $items ->
        [one] element
       *[other] elementer
    }
write-only = Kun skrivning
remove = Fjern
context-action = Konteksthandling
context-action-confirm-title = Kør “{ $name }”?
context-action-confirm-warning =
    This will run on { $items } { $items ->
        [one] item
       *[other] items
    }.
selected-items = De { $items } valgte elementer
mixed = Blandet
none = Ingen
write-execute = Skriv og kør
read-only = Kun læsning
read-execute = Læs og kør
read-write = Læs og skriv
read-write-execute = Læs, skriv og kør
keep = Behold
support = Support
progress-failed = { $percent } %, mislykkedes
pasted-image = Indsat billede
pasted-text = Indsat tekst
pasted-video = Indsat video
deleting =
    Sletter { $items } { $items ->
        [one] element
       *[other] elementer
    } fra { trash } ({ $progress })...
setting-permissions = Indstiller tilladelser for “{ $name }” til { $mode }
set-permissions = Sæt tilladelser for “{ $name }” til { $mode }
show-recents = Seneste-mappe i sidepanelet
type-to-search = Skriv for at søge
type-to-search-recursive = Søger i den aktuelle mappe og alle undermapper
type-to-search-select = Vælger den første matchende fil eller mappe
clear-recents-history = Ryd seneste historik
copy-to = Kopiér til…
delete-permanently = Slet permanent
eject = Skub ud
move-to = Flyt til…
remove-from-recents = Fjern fra seneste
reload-folder = Genindlæs mappe
copy-path = Kopiér sti
rename-confirm = Omdøb
calculate = Udregn
error = Fejl
checksum = { $kind }-kontrolsum
other-apps = Andre applikationer
related-apps = Relaterede applikationer
permanently-delete-question = Slet permanent?
permanently-delete-warning = { $target } vil blive slettet permanent. Denne handling kan ikke fortrydes.
open-with = Åbn med
favorite-path-error = Fejl ved åbning af sti
favorite-path-error-description =
    Ude af stand til at åbne "{ $path }"
    "{ $path }" Eksisterer muligvis ikke, eller du har måske ikketilladelse til at åbne den

    Ønsker du at fjerne den fra sidebjælken?
single-click = Enkelt klik for at åbne
