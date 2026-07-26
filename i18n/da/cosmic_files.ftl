cosmic-files = COSMIC Filer
empty-folder = Tom mappe
empty-folder-hidden = Tom mappe (har skjulte filer)
no-results = Ingen resultater
filesystem = Filsystem
home = Hjem
networks = Netværk
notification-in-progress = Filoperationer er igangværende.
trash = Skraldespand
recents = Seneste
undo = Fortryd
today = I dag
# Desktop view options
desktop-view-options = Indstillinger for skrivebordsudseende...
show-on-desktop = Vis på Skrivebordet
desktop-folder-content = Indhold på Skrivebordet
mounted-drives = Monterede drev
trash-folder-icon = Skraldespandsikon
icon-size-and-spacing = Ikonstørrelse og afstand
icon-size = Ikonstørrelse
# List view
name = Navn
modified = Ændret
trashed-on = Smidt ud
size = Størrelse
# Progress footer
details = Detaljer
dismiss = Afvis besked
operations-running = { $running } operationer er i gang ({ $percent }%)...
operations-running-finished = { $running } operationer er i gang ({ $percent }%), { $finished } færdiggjort...
pause = Pause
resume = Fortsæt

# Dialogs


## Compress Dialog

create-archive = Opret arkiv

## Empty Trash Dialog

empty-trash = Tøm skraldespand
empty-trash-warning = Er du sikker på du vil slette alle objekter i Skraldespanden permanent?

## Mount Error Dialog

mount-error = Kan ikke tilgå drev

## New File/Folder Dialog

create-new-file = Opret ny fil
create-new-folder = Opret ny mappe
file-name = Filnavn
folder-name = Mappenavn
file-already-exists = En fil med det navn eksisterer allerede.
folder-already-exists = En mappe med det navn eksisterer allerede.
name-hidden = Navne begyndende med "." vil blive skjult.
name-invalid = Navn kan ikke være "{ $filename }".
name-no-slashes = Navn kan ikke indeholde skråstreg.

## Open/Save Dialog

cancel = Annullér
create = Opret
open = Åbn
open-file = Åbn fil
open-folder = Åbn mappe
open-in-new-tab = Åbn i ny fane
open-in-new-window = Åbn i nyt vindue
open-item-location = Åbn placering for objekt
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
replace-title = "{ $filename }" eksisterer allerede på denne placering.
replace-warning = Vil du erstatte den med den du er ved at gemme? Hvis du erstatter den, overskriver du dens indhold.
replace-warning-operation = Vil du erstatte den? Hvis du erstatter den, overskriver du dens indhold.
original-file = Original fil
replace-with = Erstat med
apply-to-all = Anvend for alle
keep-both = Behold begge
skip = Spring over

## Set as Executable and Launch Dialog

set-executable-and-launch = Indstil som eksekverbar fil og start
set-executable-and-launch-description = Vil du indstille "{ $name }" som en eksekverbar fil og starte den?
set-and-launch = Indstil og start

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
match-desktop = Match skrivebord
dark = Mørk
light = Lys
# Context menu
add-to-sidebar = Tilføj til sidebjælke
compress = Komprimér
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
paste = Sæt ind
select-all = Vælg alt

## View

zoom-in = Zoom ind
default-size = Standardstørrelse
zoom-out = Zoom ud
view = Visning
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
