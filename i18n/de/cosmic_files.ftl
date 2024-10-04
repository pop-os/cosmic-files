cosmic-files = COSMIC Dateien
empty-folder = Leerer Ordner
empty-folder-hidden = Leerer Ordner (hat versteckte Elemente)
no-results = Keine Ergebnisse gefunden
filesystem = Dateisystem
home = Benutzerordner
networks = Netzwerke
notification-in-progress = Dateioperationen sind im Gange.
trash = Papierkorb
recents = Zuletzt benutzt
undo = Rückgängig
today = Heute

# Listenansicht
name = Name
modified = Geändert
trashed-on = In den Papierkorb verschoben
size = Größe

# Dialoge

## Komprimieren-Dialog
create-archive = Archiv erstellen

## Dialogfeld zum Leeren des Papierkorbs
empty-trash = Papierkorb leeren?
empty-trash-warning = Bist du sicher, dass du alle Elemente im Papierkorb endgültig löschen möchtest?

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

## Metadaten-Dialog
owner = Eigentümer
group = Gruppe
other = Andere
read = Lesen
write = Schreiben
execute = Ausführen

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

## Operationen
edit-history = Verlauf bearbeiten
history = Verlauf
no-history = Keine Einträge im Verlauf.
pending = Ausstehend
failed = Fehlgeschlagen
complete = Abgeschlossen
compressing = {$items} {$items ->
[one] Element wird
*[other] Elemente werden
    } von {$from} nach {$to} komprimiert
compressed = {$items} {$items ->
[one] Element wurde
*[other] Elemente wurden
    } von {$from} nach {$to} komprimiert
copy_noun = Kopie
creating = {$name} in {$parent} wird erstellt
created = {$name} in {$parent} wurde erstellt
copying = {$items} {$items ->
        [one] Element wird
        *[other] Elemente werden
    } von {$from} nach {$to} kopiert
copied = {$items} {$items ->
        [one] Element wurde
        *[other] Elemente wurden
    } von {$from} nach {$to} kopiert
emptying-trash = {trash} wird geleert
emptied-trash = {trash} geleert
extracting = {$items} {$items ->
[one] Element wird
*[other] Elemente werden
    } von {$from} nach {$to} entpackt
extracted = {$items} {$items ->
[one] Element wurde
*[other] Elemente wurden
    } von {$from} nach {$to} entpackt
moving = {$items} {$items ->
        [one] Element wird
        *[other] Elemente werden
    } von {$from} nach {$to} verschoben
moved = {$items} {$items ->
        [one] Element wurde
        *[other] Elemente wurden
    } von {$from} nach {$to} verschoben
renaming = {$from} wird in {$to} umbenannt
renamed = {$from} wurde in {$to} umbenannt
restoring = {$items} {$items ->
        [one] Element wird
        *[other] Elemente werden
    } aus dem {trash} wiederhergestellt 
restored = {$items} {$items ->
        [one] Element wurde
        *[other] Elemente wurden
    } aus dem {trash} wiederhergestellt
unknown-folder = unbekannter Ordner

## Öffnen mit
open-with = Öffnen mit
default-app = {$name} (Standard)

## Eigenschaften
properties = Eigenschaften

## Einstellungen
settings = Einstellungen

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
restore-from-trash = Aus dem Papierkorb wiederherstellen
remove-from-sidebar = Von der Seitenleiste entfernen
sort-by-name = Nach Name sortieren
sort-by-modified = Nach Änderung sortieren
sort-by-size = Nach Größe sortieren
sort-by-trashed = Nach Löschzeitpunkt sortieren

# Menü

## Datei
file = Datei
new-tab = Neuer Tab
new-window = Neues Fenster
rename = Umbenennen
menu-show-details = Details anzeigen...
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
