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
operations-running = {$running} laufende Vorgänge ({$percent} %)...
operations-running-finished = {$running} laufende Vorgänge ({$percent} %), {$finished} abgeschlossen...
pause = Pause
resume = Fortsetzen

# Dialoge

## Komprimieren-Dialog
create-archive = Archiv erstellen

## Entpacken-Dialog
extract-password-required = Passwort erforderlich

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
name-invalid = Name darf nicht „{$filename}“ sein.
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
open-with-title = Wie möchtest du „{$name}“ öffnen?
browse-store = {$store} durchsuchen

## Dauerhaft Löschen Dialog
selected-items = {$items} gewählte Objekte
permanently-delete-question = {$target} dauerhaft löschen?
delete = Löschen
permanently-delete-warning = {$target} dauerhaft löschen, {$nb_items ->
        [one] es kann
        *[other] Sie können
    } nicht wiederhergestellt werden.

# Umbenennen-Dialog
rename-file = Datei umbenennen
rename-folder = Ordner umbenennen

# Ersetzen-Dialog
replace = Ersetzen
replace-title = {$filename} existiert bereits an diesem Ort.
replace-warning = Möchtest du sie durch diejenige ersetzen, die du gerade speicherst? Beim Ersetzen wird ihr Inhalt überschrieben.
replace-warning-operation = Möchtest du sie ersetzen? Beim Ersetzen wird ihr Inhalt überschrieben.
original-file = Originaldatei
replace-with = Ersetzen mit
apply-to-all = Auf alle anwenden
keep-both = Beide behalten
skip = Überspringen

## Dialog zum Festlegen als ausführbar und starten
set-executable-and-launch = Als ausführbar festlegen und starten
set-executable-and-launch-description = Möchtest du „{$name}“ als ausführbar festlegen und starten?
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

# Kontextseiten

## Über
git-description = Git-Commit {$hash} am {$date}

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
progress = {$percent} %
progress-cancelled = {$percent} %, abgeschlossen
progress-paused = {$percent} %, pausiert
failed = Fehlgeschlagen
complete = Abgeschlossen
compressing = {$items} {$items ->
        [one] Element wird
        *[other] Elemente werden
    } von „{$from}“ nach „{$to}“ komprimiert ({$progress})...
compressed = {$items} {$items ->
        [one] Element wurde
        *[other] Elemente wurden
    } von „{$from}“ nach „{$to}“ komprimiert
copy_noun = Kopie
creating = „{$name}“ in „{$parent}“ wird erstellt
created = „{$name}“ in „{$parent}“ wurde erstellt
copying = {$items} {$items ->
        [one] Element wird
        *[other] Elemente werden
    } von „{$from}“ nach „{$to}“ kopiert ({$progress})...
copied = {$items} {$items ->
        [one] Element wurde
        *[other] Elemente wurden
    } „{$from}“ nach „{$to}“ kopiert
emptying-trash = {trash} wird geleert ({$progress})...
emptied-trash = {trash} geleert
extracting = {$items} {$items ->
        [one] Element wird
        *[other] Elemente werden
    } von „{$from}“ nach „{$to}“ entpackt ({$progress})...
extracted = {$items} {$items ->
        [one] Element wurde
        *[other] Elemente wurden
    } von „{$from}“ nach „{$to}“ entpackt
setting-executable-and-launching = „{$name}“ wird als ausführbar festgelegt und gestartet
set-executable-and-launched = „{$name}“ als ausführbar festgelegt und gestartet
moving = {$items} {$items ->
        [one] Element wird
        *[other] Elemente werden
    } von „{$from}“ nach „{$to}“ verschoben ({$progress})...
moved = {$items} {$items ->
        [one] Element wurde
        *[other] Elemente wurden
    } von „{$from}“ nach „{$to}“ verschoben
renaming = „{$from}“ wird in „{$to}“ umbenannt
renamed = „{$from}“ wurde in „{$to}“ umbenannt
restoring = {$items} {$items ->
        [one] Element wird
        *[other] Elemente werden
    } aus dem {trash} wiederhergestellt ({$progress})...
restored = {$items} {$items ->
        [one] Element wurde
        *[other] Elemente wurden
    } aus dem {trash} wiederhergestellt
unknown-folder = unbekannter Ordner
permanently-deleting = Lösche {$items} {$items ->
        [one] Objekt
        *[other] Objekte
    } dauerhaft
permanently-deleted = {$items} {$items ->
        [one] Objekt
        *[other] Objekte
    } dauerhaft gelöscht

## Öffnen mit
menu-open-with = Öffnen mit
default-app = {$name} (Standard)

## Details anzeigen
show-details = Details anzeigen
type = Typ: {$mime}
items = Elemente: {$items}
item-size = Größe: {$size}
item-created = Erstellt: {$created}
item-modified = Geändert: {$modified}
item-accessed = Zugegriffen: {$accessed}
calculating = Wird berechnet...

## Einstellungen
settings = Einstellungen
settings-show-delete-permanently = Dauerhaft löschen

### Aussehen
appearance = Aussehen
theme = Thema
match-desktop = An Desktop anpassen
dark = Dunkel
light = Hell

# Kontextmenü
add-to-sidebar = Zur Seitenleiste hinzufügen
compress = Komprimieren
extract-here = Entpacken
new-file = Neue Datei
new-folder = Neuer Ordner
open-in-terminal = Im Terminal öffnen
move-to-trash = In den Papierkorb verschieben
delete-permanently = Dauerhaft löschen...
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
