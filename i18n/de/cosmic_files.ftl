cosmic-files = COSMIC Dateien
empty-folder = Leerer Ordner
empty-folder-hidden = Leerer Ordner (hat versteckte Elemente)
no-results = Keine Ergebnisse gefunden
filesystem = Dateisystem
home = Benutzerordner
networks = Netzwerke
notification-in-progress = Dateivorgänge sind im Gange.
trash = Papierkorb
recents = Zuletzt benutzt
undo = Rückgängig
today = Heute
# Optionen für die Desktop-Ansicht
desktop-view-options = Optionen für die Desktop-Ansicht...
show-on-desktop = Auf Desktop anzeigen
desktop-folder-content = Inhalt des Desktop-Ordners
mounted-drives = Eingehängte Laufwerke
trash-folder-icon = Ordnersymbol des Papierkorbs
icon-size-and-spacing = Symbolgröße und -abstand
icon-size = Symbolgröße
grid-spacing = Rasterabstand
# Listenansicht
name = Name
modified = Geändert
trashed-on = In den Papierkorb verschoben
size = Größe
# Fortschrittsfußzeile
details = Details
dismiss = Meldung verwerfen
operations-running =
    { $running } { $running ->
        [one] laufender Vorgang
       *[other] laufende Vorgänge
    } ({ $percent } %)...
operations-running-finished =
    { $running } { $running ->
        [one] laufender Vorgang
       *[other] laufende Vorgänge
    } ({ $percent } %), { $finished } abgeschlossen...
pause = Pause
resume = Fortsetzen

# Dialoge


## Komprimieren-Dialog

create-archive = Archiv erstellen

## Entpacken-Dialog

extract-password-required = Passwort erforderlich
extract-to = Entpacken nach...
extract-to-title = In Ordner entpacken

## Dialog zum Leeren des Papierkorbs

empty-trash = Papierkorb leeren?
empty-trash-warning = Bist du sicher, dass du alle Elemente im Papierkorb endgültig löschen möchtest?

## Einhängefehler-Dialog

mount-error = Zugriff auf Laufwerk nicht möglich
# Neue(r) Datei/Ordner-Dialog
create-new-file = Neue Datei erstellen
create-new-folder = Neuen Ordner erstellen
file-name = Dateiname
folder-name = Ordnername
file-already-exists = Eine Datei mit diesem Namen existiert bereits.
folder-already-exists = Ein Ordner mit diesem Namen existiert bereits.
name-hidden = Mit „.“ beginnende Namen werden ausgeblendet.
name-invalid = Name darf nicht „{ $filename }“ sein.
name-no-slashes = Namen dürfen keine Schrägstriche enthalten.
# Öffnen/Speichern-Dialog
cancel = Abbrechen
create = Erstellen
open = Öffnen
open-file = Datei öffnen
open-folder = Ordner öffnen
open-in-new-tab = In neuem Tab öffnen
open-in-new-window = In neuem Fenster öffnen
open-item-location = Speicherort des Elements öffnen
open-multiple-files = Mehrere Dateien öffnen
open-multiple-folders = Mehrere Ordner öffnen
save = Speichern
save-file = Datei speichern

## Öffnen-mit-Dialog

open-with-title = Wie möchtest du „{ $name }“ öffnen?
browse-store = { $store } durchsuchen
other-apps = Andere Anwendungen
related-apps = Ähnliche Anwendungen

## Endgültig-löschen-Dialog

selected-items = die { $items } ausgewählten Elemente
permanently-delete-question = Endgültig löschen?
delete = Löschen
permanently-delete-warning = Bist du sicher, dass du { $target } endgültig löschen möchtest? Dies kann nicht rückgängig gemacht werden.
# Umbenennen-Dialog
rename-file = Datei umbenennen
rename-folder = Ordner umbenennen
# Ersetzen-Dialog
replace = Ersetzen
replace-title = { $filename } existiert bereits an diesem Ort.
replace-warning = Möchtest du sie durch diejenige ersetzen, die du gerade speicherst? Beim Ersetzen wird ihr Inhalt überschrieben.
replace-warning-operation = Möchtest du sie ersetzen? Beim Ersetzen wird ihr Inhalt überschrieben.
original-file = Originaldatei
replace-with = Ersetzen mit
apply-to-all = Auf alle anwenden
keep-both = Beide behalten
skip = Überspringen

## Dialog zum Festlegen als ausführbar und starten

set-executable-and-launch = Als ausführbar festlegen und starten
set-executable-and-launch-description = Möchtest du „{ $name }“ als ausführbar festlegen und starten?
set-and-launch = Festlegen und starten

## Metadaten-Dialog

open-with = Öffnen mit
owner = Eigentümer
group = Gruppe
other = Andere

### Modus 0

none = Keine

### Modus 1 (ungewöhnlich)

execute-only = Nur ausführen

### Modus 2 (ungewöhnlich)

write-only = Nur schreiben

### Modus 3 (ungewöhnlich)

write-execute = Schreiben und ausführen

### Modus 4

read-only = Nur lesen

### Modus 5

read-execute = Lesen und ausführen

### Modus 6

read-write = Lesen und schreiben

### Modus 7

read-write-execute = Lesen, schreiben und ausführen

## Fehlerdialog zum gewünschten Pfad

favorite-path-error = Fehler beim Öffnen des Verzeichnisses
favorite-path-error-description =
    „{ $path }“ kann nicht geöffnet werden.
        Möglicherweise existiert es nicht oder du hast keine Berechtigung, es zu öffnen.

        Möchtest du es aus der Seitenleiste entfernen?
remove = Entfernen
keep = Behalten

# Kontextseiten


## Über


## Netzlaufwerk hinzufügen

add-network-drive = Netzlaufwerk hinzufügen
connect = Verbinden
connect-anonymously = Anonym verbinden
connecting = Wird verbunden...
domain = Domain
enter-server-address = Serveradresse eingeben
network-drive-description =
    Serveradressen enthalten ein Protokollpräfix und eine Adresse.
    Beispiele: ssh://192.168.0.1, ftp://[2001:db8::1]

### Achte darauf, dass das Komma, das die Spalten trennt, erhalten bleibt

network-drive-schemes =
    Verfügbare Protokolle,Präfix
    AppleTalk,afp://
    File Transfer Protocol,ftp:// oder ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// oder ssh://
    WebDav,dav:// oder davs://
network-drive-error = Zugriff auf Netzlaufwerk nicht möglich
password = Passwort
remember-password = Passwort merken
try-again = Erneut versuchen
username = Benutzername

## Vorgänge

cancelled = Abgebrochen
edit-history = Verlauf bearbeiten
history = Verlauf
no-history = Keine Einträge im Verlauf.
pending = Ausstehend
progress = { $percent } %
progress-cancelled = { $percent } %, abgeschlossen
progress-paused = { $percent } %, pausiert
failed = Fehlgeschlagen
complete = Abgeschlossen
compressing =
    { $items } { $items ->
        [one] Element wird
       *[other] Elemente werden
    } von „{ $from }“ nach „{ $to }“ komprimiert ({ $progress })...
compressed =
    { $items } { $items ->
        [one] Element wurde
       *[other] Elemente wurden
    } von „{ $from }“ nach „{ $to }“ komprimiert
copy_noun = Kopie
creating = „{ $name }“ in „{ $parent }“ wird erstellt
created = „{ $name }“ in „{ $parent }“ wurde erstellt
copying =
    { $items } { $items ->
        [one] Element wird
       *[other] Elemente werden
    } von „{ $from }“ nach „{ $to }“ kopiert ({ $progress })...
copied =
    { $items } { $items ->
        [one] Element wurde
       *[other] Elemente wurden
    } „{ $from }“ nach „{ $to }“ kopiert
deleting =
    { $items } { $items ->
        [one] Element wird
       *[other] Elemente werden
    } aus dem { trash } gelöscht ({ $progress })...
deleted =
    { $items } { $items ->
        [one] Element wurde
       *[other] Elemente wurden
    } aus dem { trash } gelöscht
emptying-trash = { trash } wird geleert ({ $progress })...
emptied-trash = { trash } geleert
extracting =
    { $items } { $items ->
        [one] Element wird
       *[other] Elemente werden
    } von „{ $from }“ nach „{ $to }“ entpackt ({ $progress })...
extracted =
    { $items } { $items ->
        [one] Element wurde
       *[other] Elemente wurden
    } von „{ $from }“ nach „{ $to }“ entpackt
setting-executable-and-launching = „{ $name }“ wird als ausführbar festgelegt und gestartet
set-executable-and-launched = „{ $name }“ als ausführbar festgelegt und gestartet
setting-permissions = Berechtigungen für „{ $name }“ werden auf { $mode } festgelegt
set-permissions = Berechtigungen für „{ $name }“ auf { $mode } festlegen
moving =
    { $items } { $items ->
        [one] Element wird
       *[other] Elemente werden
    } von „{ $from }“ nach „{ $to }“ verschoben ({ $progress })...
moved =
    { $items } { $items ->
        [one] Element wurde
       *[other] Elemente wurden
    } von „{ $from }“ nach „{ $to }“ verschoben
permanently-deleting =
    { $items } { $items ->
        [one] Element wird
       *[other] Elemente werden
    } endgültig gelöscht
permanently-deleted =
    { $items } { $items ->
        [one] Element wurde
       *[other] Element wurden
    } endgültig gelöscht
renaming = „{ $from }“ wird in „{ $to }“ umbenannt
renamed = „{ $from }“ wurde in „{ $to }“ umbenannt
restoring =
    { $items } { $items ->
        [one] Element wird
       *[other] Elemente werden
    } aus dem { trash } wiederhergestellt ({ $progress })...
restored =
    { $items } { $items ->
        [one] Element wurde
       *[other] Elemente wurden
    } aus dem { trash } wiederhergestellt
unknown-folder = unbekannter Ordner

## Öffnen mit

menu-open-with = Öffnen mit
default-app = { $name } (Standard)

## Details anzeigen

show-details = Details anzeigen
type = Typ: { $mime }
items = Elemente: { $items }
item-size = Größe: { $size }
item-created = Erstellt: { $created }
item-modified = Geändert: { $modified }
item-accessed = Zugegriffen: { $accessed }
calculating = Wird berechnet...

## Einstellungen

settings = Einstellungen
single-click = Einzelklick zum Öffnen

### Aussehen

appearance = Aussehen
theme = Thema
match-desktop = An Desktop anpassen
dark = Dunkel
light = Hell

### Zum Suchen tippen

type-to-search = Zum Suchen tippen
type-to-search-recursive = Durchsucht den aktuellen Ordner und alle Unterordner
type-to-search-enter-path = Gibt den Pfad zu einem Verzeichnis oder einer Datei ein
# Kontextmenü
add-to-sidebar = Zur Seitenleiste hinzufügen
compress = Komprimieren
delete-permanently = Endgültig löschen
extract-here = Entpacken
new-file = Neue Datei...
new-folder = Neuer Ordner...
open-in-terminal = Im Terminal öffnen
move-to-trash = In den Papierkorb verschieben
restore-from-trash = Aus dem Papierkorb wiederherstellen
remove-from-sidebar = Von der Seitenleiste entfernen
sort-by-name = Nach Name sortieren
sort-by-modified = Nach Änderung sortieren
sort-by-size = Nach Größe sortieren
sort-by-trashed = Nach Löschzeitpunkt sortieren

## Desktop

change-wallpaper = Hintergrundbild ändern...
desktop-appearance = Desktop-Aussehen...
display-settings = Anzeigeeinstellungen...

# Menü


## Datei

file = Datei
new-tab = Neuer Tab
new-window = Neues Fenster
reload-folder = Ordner neu laden
rename = Umbenennen...
close-tab = Tab schließen
quit = Beenden

## Bearbeiten

edit = Bearbeiten
cut = Ausschneiden
copy = Kopieren
paste = Einfügen
select-all = Alles auswählen

## Ansicht

zoom-in = Vergrößern
default-size = Standardgröße
zoom-out = Verkleinern
view = Ansicht
grid-view = Rasteransicht
list-view = Listenansicht
show-hidden-files = Versteckte Dateien anzeigen
list-directories-first = Verzeichnisse zuerst auflisten
gallery-preview = Galerie-Vorschau
menu-settings = Einstellungen...
menu-about = Über COSMIC Dateien...

## Sortieren

sort = Sortieren
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Neueste zuerst
sort-oldest-first = Älteste zuerst
sort-smallest-to-largest = Kleinste bis größte
sort-largest-to-smallest = Größte bis kleinste
repository = Repository
