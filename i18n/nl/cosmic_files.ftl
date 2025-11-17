cosmic-files = COSMIC Bestanden
empty-folder = Lege map
empty-folder-hidden = Lege map (met verborgen bestanden)
no-results = Geen resultaten gevonden
filesystem = Bestandssysteem
home = Home
networks = Netwerk
notification-in-progress = Bestanden worden nog bewerkt
trash = Prullenbak
recents = Recente bestanden
undo = Herstel
today = Vandaag
# Desktop view options
desktop-view-options = Opties voor bureaubladweergave…
show-on-desktop = Op bureaublad weergeven
desktop-folder-content = Bestanden in de Bureablad map
mounted-drives = Aangekoppelde schijven
trash-folder-icon = Pictogram van de map “Prullenbak”
icon-size-and-spacing = Pictogram grootte en tussenruimte
icon-size = Pictogramgrootte
grid-spacing = Rastergrootte
# List view
name = Naam
modified = Gewijzigd
trashed-on = Verwijderd op
size = Grootte
# Progress footer
details = Details
dismiss = Negeer bericht
operations-running =
    { $running } { $running ->
        [one] bewerking
       *[other] bewerkingen
    } uitvoeren ({ $percent }%)…
operations-running-finished =
    { $running } { $running ->
        [one] bewerking
       *[other] bewerkingen
    } uitvoeren ({ $percent }%), { $finished } voltooid...
pause = Pauzeren
resume = Hervatten

# Dialogs


## Compress Dialog

create-archive = Archiefbestand maken

## Extract Dialog

extract-password-required = Wachtwoord vereist
extract-to = Uitpakken naar...
extract-to-title = Uitpakken naar map

## Empty Trash Dialog

empty-trash = Leeg prullenbak
empty-trash-warning = Weet u zeker dat u alles in de prullenbak permanent wilt verwijderen?

## Mount Error Dialog

mount-error = Geen toegang tot schijf

## New File/Folder Dialog

create-new-file = Nieuw bestand aanmaken
create-new-folder = Nieuwe map aanmaken
file-name = Bestandsnaam
folder-name = Mapnaam
file-already-exists = Er bestaat al een bestand met deze naam.
folder-already-exists = Er bestaat al een map met deze naam.
name-hidden = Namen die met '.' beginnen worden verborgen.
name-invalid = De naam '{ $filename }' is niet geldig.
name-no-slashes = De naam mag geen slash '/' bevatten.

## Open/Save Dialog

cancel = Annuleer
create = Aanmaken
open = Openen
open-file = Open een bestand
open-folder = Open een map
open-in-new-tab = Open in nieuw tabblad
open-in-new-window = Open in nieuw venster
open-item-location = Open locatie van item
open-multiple-files = Meerdere bestanden openen
open-multiple-folders = Meerdere mappen openen
save = Sla op
save-file = Bestand opslaan

## Open With Dialog

open-with-title = Hoe wilt u '{ $name }' openen?
browse-store = Verken { $store }
other-apps = Andere toepassingen
related-apps = Gerelateerde toepassingen

## Permanently delete Dialog

selected-items = de { $items } geselecteerde items
permanently-delete-question = Permanent verwijderen?
delete = Verwijder
permanently-delete-warning = Weet u zeker dat u { $target } permanent wilt verwijderen? Dit kan niet ongedaan worden gemaakt.

## Rename Dialog

rename-file = Bestand hernoemen
rename-folder = Map hernoemen

## Replace Dialog

replace = Vervang
replace-title = '{ $filename }' bestaat al in deze map.
replace-warning = Wilt u het bestand vervangen door de nieuwe versie? Dit zal de bestaande inhoud overschrijven.
replace-warning-operation = Wilt u het bestand vervangen? De bestaande inhoud wordt overschreven!
original-file = Oorspronkelijk bestand
replace-with = Vervangen door
apply-to-all = Op alles toepassen
keep-both = Beide behouden
skip = Overslaan

## Set as Executable and Launch Dialog

set-executable-and-launch = Uitvoerbaar maken en dan starten
set-executable-and-launch-description = Wilt u '{ $name }' uitvoerbaar maken en dan starten?
set-and-launch = Uitvoerbaar maken en starten

## Metadata Dialog

open-with = Openen met
owner = Eigenaar
group = Groep
other = Anderen

### Mode 0

none = Geen rechten

### Mode 1 (unusual)

execute-only = Alleen uitvoeren

### Mode 2 (unusual)

write-only = Alleen schrijven

### Mode 3 (unusual)

write-execute = Schijven en uitvoeren

### Mode 4

read-only = Alleen lezen

### Mode 5

read-execute = Lezen en uitvoeren

### Mode 6

read-write = Lezen en schrijven

### Mode 7

read-write-execute = Lezen, schrijven en uitvoeren

## Favorite Path Error Dialog

favorite-path-error = Fout bij het openen van de map
favorite-path-error-description =
    Kon de map '{ $path }' niet openen.
    De map bestaat mogelijk niet of u heeft geen toestemming om die te openen.

    Wilt u de map uit de favorieten verwijderen?
remove = Verwijder
keep = Behouden

# Context Pages


## About


## Add Network Drive

add-network-drive = Netwerkschijf toevoegen
connect = Verbind
connect-anonymously = Anoniem verbinden
connecting = Verbinding maken...
domain = Domein
enter-server-address = Serveradres invoeren
network-drive-description =
    Serveradressen bestaan uit protocolvoorvoegsel en netwerkadres.
    Voorbeelden: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Beschikbare protocollen,Voorvoegsel
    AppleTalk,afp://
    File Transfer Protocol,ftp:// of ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// of ssh://
    WebDav,dav:// of davs://
network-drive-error = Geen toegang tot de netwerkschijf
password = Wachtwoord
remember-password = Wachtwoord onthouden
try-again = Opnieuw proberen
username = Gebruikersnaam

## Operations

cancelled = Geannuleerd
edit-history = Geschiedenis bewerken
history = Geschiedenis
no-history = Geen items in de geschiedenis.
pending = In afwachting
progress = { $percent }%
progress-cancelled = { $percent }%, geannuleerd
progress-paused = { $percent }%, gepauzeerd
failed = Mislukt
complete = Voltooid
compressing =
    { $items }  { $items ->
        [one] bestand
       *[other] bestanden
    } van '{ $from }' naar '{ $to }' comprimeren ({ $progress })...
compressed =
    { $items }  { $items ->
        [one] bestand
       *[other] bestanden
    } gecomprimeerd van '{ $from }' naar '{ $to }'
copy_noun = Kopie
creating = '{ $name }' in '{ $parent }' aanmaken
created = '{ $name }' in '{ $parent }' aangemaakt
copying =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } van '{ $from }' naar '{ $to }' kopiëren ({ $progress })...
copied =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } gekopieerd van '{ $from }' naar '{ $to }'
deleting =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } uit { trash } verwijderen ({ $progress })...
deleted =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } verwijderd uit { trash }
emptying-trash = { trash } recyclen ({ $progress })...
emptied-trash = { trash } gerecycled
extracting =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } van '{ $from }' naar '{ $to }' uitpakken ({ $progress })...
extracted =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } uitgepakt van '{ $from }' naar '{ $to }'
setting-executable-and-launching = '{ $name }' uitvoerbaar maken en starten
set-executable-and-launched = '{ $name }' uitvoerbaar gemaakt en gestart
setting-permissions = Rechten voor '{ $name }' wijzigen in '{ $mode }'
set-permissions = Rechten voor '{ $name }' gewijzigd in '{ $mode }'
moving =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } van '{ $from }' naar '{ $to }' verplaatsen ({ $progress })...
moved =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } verplaatst van '{ $from }' naar '{ $to }'
permanently-deleting =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } premanent verwijderen
permanently-deleted =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } permanent verwijderd
renaming = '{ $from }' als '{ $to }' hernoemen
renamed = '{ $from }' als '{ $to }' hernoemd
restoring =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } uit { trash } terugzetten ({ $progress })...
restored =
    { $items } { $items ->
        [one] bestand
       *[other] bestanden
    } uit { trash } teruggezet
unknown-folder = Onbekende map

## Open with

menu-open-with = Openen met...
default-app = { $name } (standaard)

## Show details

show-details = Details weergeven
type = Type: { $mime }
items = Bestanden: { $items }
item-size = Grootte: { $size }
item-created = Aangemaakt op: { $created }
item-modified = Bewerkt op: { $modified }
item-accessed = Geopend op: { $accessed }
calculating = Wordt berekend...

## Settings

settings = Instellingen
single-click = Een keer klikken om te openen

### Appearance

appearance = Weergave
theme = Thema
match-desktop = Systeemstandaard
dark = Donker
light = Licht

### Type to Search

type-to-search = Typ om te zoeken
type-to-search-recursive = In deze map en alle onderliggende mappen zoeken
type-to-search-enter-path = Naar de bestandslocatie of -naam zoeken
# Context menu
add-to-sidebar = Favoriet aan zijbalk toevoegen
compress = Comprimeren
delete-permanently = Permanent verwijderen
extract-here = Uitpakken
new-file = Nieuw bestand...
new-folder = Nieuwe map...
open-in-terminal = Openen in terminal
move-to-trash = Naar prullenbak verplaatsen
restore-from-trash = Uit prullenbak terugzetten
remove-from-sidebar = Favoriet uit zijbalk verwijderen
sort-by-name = Sorteren op naam
sort-by-modified = Sorteren op laatst bewerkt
sort-by-size = Sorteren op grootte
sort-by-trashed = Sorteren op tijdstip van verwijderen

## Desktop

change-wallpaper = Schermachtergrond wijzigen...
desktop-appearance = Bureaublad uiterlijk...
display-settings = Beeldschermbeheer...

# Menu


## File

file = Bestand
new-tab = Nieuw tabblad
new-window = Nieuw venster
reload-folder = Opnieuw laden
rename = Hernoemen...
close-tab = Sluit tabblad
quit = Sluit

## Edit

edit = Bewerk
cut = Knip
copy = Kopieer
paste = Plak
select-all = Selecteer alles

## View

zoom-in = Zoom in
default-size = Standaardgrootte
zoom-out = Zoom uit
view = Aanzicht
grid-view = Rasterweergave
list-view = Lijstweergave
show-hidden-files = Verborgen bestanden tonen
list-directories-first = Mappen bovenaan weergeven
gallery-preview = Galerijweergave
menu-settings = Instellingen...
menu-about = Over COSMIC Bestanden...

## Sort

sort = Sorteren
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Nieuwste bovenaan
sort-oldest-first = Oudste bovenaan
sort-smallest-to-largest = Van klein naar groot
sort-largest-to-smallest = Van groot naar klein
progress-failed = { $percent }%, mislukt
eject = Uitwerpen
support = Ondersteuning
removing-from-recents =
    { $items } { $items ->
        [one] item
       *[other] items
    } van { recents }
