cosmic-files = COSMIC bestandsbeheerder
empty-folder = Lege map
empty-folder-hidden = Lege map (met verborgen bestanden)
no-results = Geen resultaten gevonden
filesystem = Bestandssysteem
home = Gebruikersmap
networks = Netwerken
notification-in-progress = Bestanden worden momenteel bewerkt.
trash = Prullenbak
recents = Recente bestanden
undo = Ongedaan maken
today = Vandaag

# Desktop view options
desktop-view-options = Opties voor bureaubladweergave
show-on-desktop = Op bureaublad weergeven
desktop-folder-content = Bestanden in Bureaublad
mounted-drives = Gekoppelde schijven
trash-folder-icon = Prullenbakicoon
icon-size-and-spacing = Grootte en ruimte tussen iconen
icon-size = Icoon grootte

# List view
name = Naam
modified = Bewerkt
trashed-on = Tijd van verwijderen
size = Grootte

# Progress footer
details = Details
dismiss = Bericht negeren
operations-running = {$running} bewerkingen worden uitgevoerd ({$percent}%)...
operations-running-finished = {$running} bewerkingen worden uitgevoerd ({$percent}%), {$finished} voltooid...
pause = Pauze
resume = Hervatten

# Dialogs

## Compress Dialog
create-archive = Maak een archiefbestand

## Empty Trash Dialog
empty-trash = Prullenbak legen?
empty-trash-warning = Weet u zeker dat u alles in de prullenbak permanent wilt verwijderen?

## Mount Error Dialog
mount-error = Toegang tot schijf niet mogelijk

## New File/Folder Dialog
create-new-file = Nieuw bestand aanmaken
create-new-folder = Nieuwe map aanmaken
file-name = Bestandsnaam
folder-name = Mapnaam
file-already-exists = Er bestaat al een bestand met deze naam.
folder-already-exists = Er bestaat al een map met deze naam.
name-hidden = Namen die met '.' beginnen worden verborgen.
name-invalid = De naam '{$filename}' is niet geldig.
name-no-slashes = De naam mag geen schuine strepen bevatten.

## Open/Save Dialog
cancel = Annuleren
create = Aanmaken
open = Openen
open-file = Bestand openen
open-folder = Map openen
open-in-new-tab = Open in nieuw tabblad
open-in-new-window = Open in nieuw venster
open-item-location = Open locatie van item
open-multiple-files = Meerdere bestanden openen
open-multiple-folders = Meerdere mappen openen
save = Opslaan
save-file = Bestand opslaan

## Open With Dialog
open-with-title = Hoe wilt u '{$name}' openen?
browse-store = Verken {$store}

## Rename Dialog
rename-file = Bestand hernoemen
rename-folder = Map hernoemen

## Replace Dialog
replace = Vervangen
replace-title = '{$filename}' bestaat al op deze locatie.
replace-warning = Wilt u het bestand vervangen door de nieuwe versie? Dit zal de bestaande inhoud overschrijven.
replace-warning-operation = Wilt u het bestand vervangen? Bestaande inhoud wordt overschreven!
original-file = Oorspronkelijk bestand
replace-with = Vervangen door
apply-to-all = Op alles toepassen
keep-both = Beide behouden
skip = Overslaan

## Set as Executable and Launch Dialog
set-executable-and-launch = Bestand uitvoerbaar maken en dan openen
set-executable-and-launch-description = Wilt u '{$name}' uitvoerbaar maken en dan openen?
set-and-launch = Maak uitvoerbaar en open

## Metadata Dialog
owner = Eigenaar
group = Groep
other = Anderen
read = Lezen
write = Schrijven
execute = Uitvoeren

# Context Pages

## About
git-description = Git commit {$hash} op {$date}

## Add Network Drive
add-network-drive = Netwerkschijf toevoegen
connect = Verbinden
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
progress = {$percent}%
progress-cancelled = {$percent}%, geannuleerd
progress-paused = {$percent}%, gepauzeerd
failed = Mislukt
complete = Voltooid
compressing = { $items}  {$items -> 
        [one] bestand wordt
        *[other] bestanden worden
    } van '{$from}' naar '{$to}' gecomprimeerd ({$progress})...
compressed = { $items}  {$items -> 
        [one] bestand
        *[other] bestanden
    } gecomprimeerd van '{$from}' naar '{$to}'
copy_noun = Kopie
creating = '{$name}' in '{$parent}' aanmaken
created = '{$name}' in '{$parent}' aangemaakt
copying = {$items} {$items ->
        [one] bestand wordt
        *[other] bestanden worden
    } van '{$from}' naar '{$to}' gekopieerd ({$progress})...
emptying-trash = {trash} wordt geleegd ({$progress})...
emptied-trash = {trash} geleegd
extracting = {$items} {$items -> 
        [one] bestand wordt
        *[other] bestanden worden
    } van '{$from}' naar '{$to}' uitgepakt ({$progress})...
extracted = {$items} {$items ->
        [one] bestand
        *[other] bestanden
    } uitgepakt van '{$from}' naar '{$to}'
setting-executable-and-launching = '{$name}' wordt uitvoerbaar gemaakt en geopend
set-executable-and-launched = '{$name}' uitvoerbaar maken en openen
moving = {$items} {$items ->
        [one] bestand wordt
        *[other] bestanden worden
    } van '{$from}' naar '{$to}' verplaatst ({$progress})...
moved = {$items} {$items ->
        [one] bestand
        *[other] bestanden
    } verplaatst van '{$form}' naar '{$to}'
renaming = '{$from}' als '{$to}' hernoemen
renamed = '{$form}' als '{$to}' hernoemd
restoring = {$items} {$items ->
        [one] bestand wordt
        *[other] bestanden worden
    } uit {trash} teruggezet ({$progress})...
restored = {$items} {$items ->
        [one] bestand
        *[other] bestanden
    } uit {trash} teruggezet
unknown-folder = Onbekende map

## Open with
open-with = Openen met...
default-app = {$name} (standaard)

## Show details
show-details = Details weergeven
type = Type: {$mime}
items = Bestanden: {$items}
item-size = Grootte: {$size}
item-created = Aangemaakt op: {$created}
item-modified = Bewerkt op: {$modified}
item-accessed = Geopend op: {$accessed}
calculating = Wordt berekend...

## Settings
settings = Instellingen

### Appearance
appearance = Uiterlijk
theme = Thema
match-desktop = Systeemstandaard
dark = Donker
light = Licht

# Context menu
add-to-sidebar = Aan de zijbalk toevoegen
compress = Comprimeren
extract-here = Uitpakken
new-file = Nieuw bestand...
new-folder = Nieuwe map...
open-in-terminal = Openen in terminal
move-to-trash = Naar prullenbak verplaatsen
restore-from-trash = Uit prullenbak terugzetten
remove-from-sidebar = Uit de zijbalk verwijderen
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
rename = Hernoemen...
close-tab = Tabblad sluiten
quit = Sluiten

## Edit
edit = Bewerken
cut = Knippen
copy = Kopiëren
paste = Plakken
select-all = Alles selecteren

## View
zoom-in = Inzoomen
default-size = Zoomniveau terugzetten
zoom-out = Uitzoomen
view = Aanzicht
grid-view = Rasterweergave
list-view = Lijstweergave
show-hidden-files = Verborgen bestanden tonen
list-directories-first = Mappen bovenaan weergeven
gallery-preview = Galerijweergave
menu-settings = Instellingen...
menu-about = Over COSMIC bestandsbeheerder...

## Sort
sort = Sorteren
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Nieuwste bovenaan
sort-oldest-first = Oudste bovenaan
sort-smallest-to-largest = Van klein naar groot
sort-largest-to-smallest = Van groot naar klein
