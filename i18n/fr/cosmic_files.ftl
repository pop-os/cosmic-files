cosmic-files = Fichiers COSMIC
empty-folder = Dossier vide
empty-folder-hidden = Dossier vide (contient des éléments cachés)
no-results = Aucun résultat trouvé
filesystem = Système de fichiers
home = Dossier personnel
networks = Réseaux
notification-in-progress = Des opérations sur des fichiers sont en cours.
trash = Corbeille
recents = Récents
undo = Annuler
today = Aujourd'hui

# Desktop view options
desktop-view-options = Options d'affichage du bureau...
show-on-desktop = Afficher sur le bureau
desktop-folder-content = Contenu du dossier du bureau
mount-drives = Lecteurs montés
trash-folder-icon = Icône du dossier Corbeille
icon-size-and-spacing = Taille et espacement des icônes
icon-size = Taille des icônes

# List view
name = Nom
modified = Modifié
trashed-on = Mis à la corbeille
size = Taille

# Dialogs

## Compress Dialog
create-archive = Créer une archive

## Empty Trash Dialog
empty-trash = Vider la corbeille
empty-trash-warning = Êtes-vous sûr de vouloir supprimer définitivement tous les éléments de la corbeille ?

## Mount Error Dialog
mount-error = Impossible d'accéder au lecteur

## New File/Folder Dialog
create-new-file = Créer un nouveau fichier
create-new-folder = Créer un nouveau dossier
file-name = Nom du fichier
folder-name = Nom du dossier
file-already-exists = Un fichier portant ce nom existe déjà.
folder-already-exists = Un dossier portant ce nom existe déjà.
name-hidden = Les noms commençant par "." seront cachés.
name-invalid = Le nom ne peut pas être "{$filename}".
name-no-slashes = Le nom ne peut pas contenir de slashs.

## Open/Save Dialog
cancel = Annuler
create = Créer
open = Ouvrir
open-file = Ouvrir le fichier
open-folder = Ouvrir le dossier
open-in-new-tab = Ouvrir dans un nouvel onglet
open-in-new-window = Ouvrir dans une nouvelle fenêtre
open-item-location = Ouvrir l'emplacement de l'élément
open-multiple-files = Ouvrir plusieurs fichiers
open-multiple-folders = Ouvrir plusieurs dossiers
save = Enregistrer
save-file = Enregistrer fichier

## Open With Dialog
open-with-title = Comment souhaitez-vous ouvrir "{$name}" ?
browse-store = Parcourir {$store}

## Rename Dialog
rename-file = Renommer le fichier
rename-folder = Renommer le dossier

## Replace Dialog
replace = Remplacer
replace-title = {$filename} existe déjà à cet endroit.
replace-warning = Voulez-vous remplacer ce fichier par celui que vous enregistrez ? Cela écrasera son contenu.
replace-warning-operation = Voulez-vous remplacer ce fichier ? Cela écrasera son contenu.
original-file = Fichier d'origine
replace-with = Remplacer par
apply-to-all = Appliquer à tous
keep-both = Conserver les deux
skip = Ignorer

## Set as Executable and Launch Dialog
set-executable-and-launch = Définir comme exécutable et lancer
set-executable-and-launch-description = Voulez-vous définir "{$name}" comme exécutable et le lancer ?
set-and-launch = Définir et lancer

## Metadata Dialog
owner = Propriétaire
group = Groupe
other = Autre
read = Lecture
write = Écriture
execute = Exécution

# Context Pages

## About
git-description = Git commit {$hash} le {$date}

## Add Network Drive
add-network-drive = Ajouter un lecteur réseau
connect = Connecter
connect-anonymously = Connecter anonymement
connecting = Connection en cours...
domain = Domaine
enter-server-address = Entrez l'adresse du serveur
network-drive-description =
    Les adresses de serveur incluent un préfixe de protocole et une adresse.
    Exemples: ssh://192.168.0.1, ftp://[2001:db8::1]
### Make sure to keep the comma which separates the columns
network-drive-schemes =
    Protocoles disponibles,Préfixe
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = Impossible d'accéder au lecteur réseau
password = Mot de passe
remember-password = Se souvenir du mot de passe
try-again = Essayer à nouveau
username = Nom d'utilisateur

## Operations
edit-history = Modifier l'historique
history = Historique
no-history = Aucun élément dans l'historique.
pending = En attente
failed = Échoué
complete = Terminé
compressing = Compression de {$items} {$items ->
        [one] élément
        *[other] éléments
    } depuis {$from} vers {$to}
compressed = {$items} {$items ->
        [one] élément compressé
        *[other] éléments compressés
    } depuis {$from} vers {$to}
copy_noun = Copier
creating = Création de {$name} dans {$parent}
created = {$name} créé dans {$parent}
copying = Copie de {$items} {$items ->
        [one] élément
        *[other] éléments
    } depuis {$from} vers {$to}
copied = {$items} {$items ->
        [one] élément copié
        *[other] éléments copiés
    } depuis {$from} vers {$to}
emptying-trash = {trash} en cours de nettoyage
emptied-trash = {trash} vidée
extracting = Extraction de {$items} {$items ->
        [one] élément
        *[other] éléments
    } depuis {$from} vers {$to}
extracted = {$items} {$items ->
        [one] élément extrait
        *[other] éléments extraits
    } depuis {$from} vers {$to}
setting-executable-and-launching = Paramétrage de "{$name}" comme exécutable et prêt à être lancé
set-executable-and-launched = Définir "{$name}" comme exécutable et le lancer
moving = Déplacement de {$items} {$items ->
        [one] élément
        *[other] éléments
    } depuis {$from} vers {$to}
moved = {$items} {$items ->
        [one] élément déplacé
        *[other] éléments déplacés
    } depuis {$from} vers {$to}
renaming = Renommage de {$from} en {$to}
renamed = {$from} renommé en {$to}
restoring = Restauration de {$items} {$items ->
        [one] élément
        *[other] éléments
    } depuis la {trash}
restored = {$items} {$items ->
        [one] élément restauré
        *[other] éléments restaurés
    } depuis la {trash}
unknown-folder = Dossier inconnu

## Open with
open-with = Ouvrir avec
default-app = {$name} (défaut)

## Show details
show-details = Afficher les détails

## Settings
settings = Paramètres

### Appearance
appearance = Apparence
theme = Thème
match-desktop = Assortir au bureau
dark = Sombre
light = Clair

# Context menu
add-to-sidebar = Ajouter à la barre latérale
compress = Compresser
extract-here = Extraire
new-file = Nouveau fichier
new-folder = Nouveau dossier
open-in-terminal = Ouvrir dans le terminal
move-to-trash = Déplacer vers la corbeille
restore-from-trash = Restaurer depuis la corbeille
remove-from-sidebar = Supprimer de la barre latérale
sort-by-name = Trier par nom
sort-by-modified = Trier par modification
sort-by-size = Trier par taille
sort-by-trashed = Trier par éléments supprimés

## Desktop
change-wallpaper = Changer le fond d'écran...
desktop-appearance = Apparence du bureau...
display-settings = Paramètres d'affichage...

# Menu

## File
file = Fichier
new-tab = Nouvel onglet
new-window = Nouvelle fenêtre
rename = Renommer...
menu-show-details = Afficher les détails...
close-tab = Fermer l'onglet
quit = Quitter

## Edit
edit = Modifier
cut = Couper
copy = Copier
paste = Coller
select-all = Sélectionner tout

## View
zoom-in = Zoomer
default-size = Taille par défaut
zoom-out = Dézoomer
view = Vue
grid-view = Vue en grille
list-view = Vue en liste
show-hidden-files = Afficher les fichiers cachés
list-directories-first = Lister les répertoires en premier
menu-settings = Paramètres...
menu-about = À propos de Fichiers COSMIC...

## Sort
sort = Trier
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Le plus récent en premier
sort-oldest-first = Le plus ancien en premier
sort-smallest-to-largest = Du plus petit au plus grand
sort-largest-to-smallest = Du plus grand au plus petit
