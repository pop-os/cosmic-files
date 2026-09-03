cosmic-files = कॉस्मिक फाइल्स
empty-folder = खाली फ़ोल्डर
empty-folder-hidden = खाली फ़ोल्डर (अदृश्य आइटम शामिल हैं)
no-results = कोई परिणाम नहीं
filesystem = फाइल सिस्टम
home = होम
networks = नेटवर्क्स
notification-in-progress = फ़ाइल संचालन प्रगति पर हैं
trash = कचरा
recents = हाल के
undo = पूर्ववत करें
today = आज
# Desktop view options
desktop-view-options = डेस्कटॉप दृश्य विकल्प...
show-on-desktop = डेस्कटॉप पर दिखाएं
desktop-folder-content = डेस्कटॉप फ़ोल्डर सामग्री
mounted-drives = माउंट किए गए ड्राइव
trash-folder-icon = कचरा फ़ोल्डर आइकन
icon-size-and-spacing = आइकन आकार और अंतर
icon-size = आइकन आकार
# List view
name = नाम
modified = संशोधित तिथि
trashed-on = कचरे में डालने की तिथि
size = आकार

# Dialogs


## Compress Dialog

create-archive = संग्रह बनाएँ

## Empty Trash Dialog

empty-trash = रद्दी साफ़ करें
empty-trash-warning = रद्दी फ़ोल्डर में मौजूद आइटम स्थायी रूप से हटा दिए जाएंगे

## New File/Folder Dialog

create-new-file = नई फाइल बनाएँ
create-new-folder = नया फ़ोल्डर बनाएँ
file-name = फाइल का नाम
folder-name = फ़ोल्डर का नाम
file-already-exists = उस नाम की फ़ाइल पहले से मौजूद है
folder-already-exists = उस नाम का फ़ोल्डर पहले से मौजूद है
name-hidden = "." से शुरू होने वाले नाम छिपे होंगे
name-invalid = नाम "{ $filename }" नहीं हो सकता
name-no-slashes = नाम में स्लैश नहीं हो सकते

## Open/Save Dialog

cancel = रद्द करें
create = बनाएँ
open = खोलें
open-file = फ़ाइल खोलें
open-folder = फ़ोल्डर खोलें
open-in-new-tab = नई टैब में खोलें
open-in-new-window = नई विंडो में खोलें
open-item-location = आइटम का स्थान खोलें
open-multiple-files = कई फ़ाइलें खोलें
open-multiple-folders = कई फ़ोल्डर खोलें
save = सहेजें
save-file = फ़ाइल सहेजें

## Open With Dialog

open-with-title = "{ $name }" को कैसे खोलना चाहेंगे?
browse-store = { $store } में ब्राउज़ करें

## Rename Dialog

rename-file = फाइल का नाम बदलें
rename-folder = फ़ोल्डर का नाम बदलें

## Replace Dialog

replace = बदलें
replace-title = { $filename } पहले से इस स्थान पर मौजूद है
replace-warning = क्या आप इसे प्रतिस्थापित करना चाहते हैं? यदि प्रतिस्थापित किया गया, तो मौजूदा फ़ाइल को ओवरराइट किया जाएगा।
replace-warning-operation = क्या आप इसे बदलना चाहते हैं? प्रतिस्थापित करने पर मौजूदा फ़ाइल ओवरराइट हो जाएगी।
original-file = मूल फ़ाइल
replace-with = इसके साथ प्रतिस्थापित करें
apply-to-all = सभी पर लागू करें
keep-both = दोनों रखें
skip = छोड़ें

## Set as Executable and Launch Dialog

set-executable-and-launch = निष्पादन योग्य के रूप में सेट करें और लॉन्च करें
set-executable-and-launch-description = क्या आप निष्पादन योग्य के रूप में "{ $name }" सेट करना चाहते हैं और इसे लॉन्च करते हैं?
set-and-launch = सेट करें और लॉन्च करें

## Metadata Dialog

owner = मालिक
group = समूह
other = अन्य

# Context Pages


## About


## Add Network Drive

add-network-drive = नेटवर्क ड्राइव जोड़ें
connect = कनेक्ट करें
connect-anonymously = गुमनाम रूप से कनेक्ट करें
connecting = कनेक्ट हो रहा है...
domain = डोमेन
enter-server-address = सर्वर का पता दर्ज करें
network-drive-description =
    सर्वर पते प्रोटोकॉल उपसर्ग और पते सहित होते हैं।
    उदाहरण: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    उपलब्ध प्रोटोकॉल,उपसर्ग
    एप्पलटॉक,afp://
    फ़ाइल ट्रांसफ़र प्रोटोकॉल,ftp:// या ftps://
    नेटवर्क फ़ाइल सिस्टम,nfs://
    सर्वर संदेश ब्लॉक,smb://
    SSH फ़ाइल ट्रांसफ़र प्रोटोकॉल,sftp:// या ssh://
    वेबDAV,dav:// या davs://
network-drive-error = नेटवर्क ड्राइव तक पहुंचने में असमर्थ
password = पासवर्ड
remember-password = पासवर्ड याद रखें
try-again = फिर से कोशिश करें
username = उपयोगकर्ता नाम

## Operations

edit-history = संपादन इतिहास
history = इतिहास
no-history = इतिहास में कोई आइटम नहीं
pending = लंबित
failed = विफल
complete = पूर्ण
compressing =
    संपीड़ित किया जा रहा है { $items } { $items ->
        [one] आइटम
       *[other] आइटम
    } से { $from } तक { $to }
compressed =
    संपीड़ित किया गया { $items } { $items ->
        [one] आइटम
       *[other] आइटम
    } से { $from } तक { $to }
copy_noun = नकल
creating = { $parent } में { $name } बनाया जा रहा है
created = { $parent } में { $name } बनाया गया
copying =
    { $items } { $items ->
        [one] आइटम
       *[other] आइटम
    } से { $from } तक { $to } नकल की जा रही है
copied =
    { $items } { $items ->
        [one] आइटम
       *[other] आइटम
    } से { $from } तक { $to } नकल की गई
emptying-trash = कचरा खाली किया जा रहा है
emptied-trash = कचरा खाली किया गया
extracting =
    { $items } { $items ->
        [one] आइटम
       *[other] आइटम
    } से { $from } तक { $to } निकाला जा रहा है
extracted =
    { $items } { $items ->
        [one] आइटम
       *[other] आइटम
    } से { $from } तक { $to } निकाला गया
setting-executable-and-launching = "{ $name }" को कार्यान्वयन के रूप में सेट किया जा रहा है और लॉन्च किया जा रहा है
set-executable-and-launched = "{ $name }" को कार्यान्वयन के रूप में सेट किया गया है और लॉन्च किया गया है
moving =
    { $items } { $items ->
        [one] आइटम
       *[other] आइटम
    } से { $from } तक { $to } स्थानांतरित किया जा रहा है
moved =
    { $items } { $items ->
        [one] आइटम
       *[other] आइटम
    } से { $from } तक { $to } स्थानांतरित किया गया
renaming = { $from } से { $to } तक नाम बदला जा रहा है
renamed = { $from } से { $to } तक नाम बदला गया
restoring =
    { $items } { $items ->
        [one] आइटम
       *[other] आइटम
    } को कचरे से पुनर्स्थापित किया जा रहा है
restored =
    { $items } { $items ->
        [one] आइटम
       *[other] आइटम
    } को कचरे से पुनर्स्थापित किया गया
unknown-folder = अज्ञात फ़ोल्डर

## Open with

menu-open-with = इसके साथ खोलें
default-app = { $name } (डिफ़ॉल्ट)
search-application = ऐप के नाम से खोजें

## Show details

show-details = विवरण दिखाएँ

## Settings

settings = सेटिंग्स

### Appearance

appearance = रुप-रंग
theme = प्रसंग
match-desktop = डेस्कटॉप से मेल खाएँ
dark = डार्क
light = लाइट
# Context menu
add-to-sidebar = साइडबार में जोड़ें
compress = संपीड़ित करें
extract-here = यहाँ निकालें
new-file = नई फ़ाइल...
new-folder = नया फ़ोल्डर...
open-in-terminal = टर्मिनल में खोलें
move-to-trash = कचरे में भेजें
restore-from-trash = कचरे से पुनर्स्थापित करें
remove-from-sidebar = साइडबार से निकालें
sort-by-name = नाम से क्रमबद्ध करें
sort-by-modified = संशोधित तिथि द्वारा क्रमबद्ध करें
sort-by-size = आकार द्वारा क्रमबद्ध करें
sort-by-trashed = कचरे में डालने की तिथि द्वारा क्रमबद्ध करें

## Desktop

change-wallpaper = वॉलपेपर बदलें...
desktop-appearance = डेस्कटॉप रूप...
display-settings = डिस्प्ले सेटिंग्स...

# Menu


## File

file = फ़ाइल
new-tab = नया टैब
new-window = नई विंडो
rename = नाम बदलें...
close-tab = टैब बंद करें
quit = बाहर जाएँ

## Edit

edit = संपादन
cut = काटें
copy = कॉपी करें
paste = चिपकाएँ
select-all = सभी चुनें

## View

zoom-in = बड़ा करें
default-size = मूल आकार
zoom-out = छोटा करें
view = दृश्य
grid-view = ग्रिड दृश्य
list-view = सूची दृश्य
show-hidden-files = छिपी हुई फाइलें दिखाएँ
list-directories-first = सबसे पहले डाइरेक्ट्री दिखाएँ
menu-settings = सेटिंग्स..।
menu-about = कॉस्मिक फाइल्स के बारे में...

## Sort

sort = क्रमबद्ध करें
sort-a-z = अ-ह क्रम में क्रमबद्ध करें
sort-z-a = ह-अ क्रम में क्रमबद्ध करें
sort-newest-first = नए से पुराने
sort-oldest-first = पुराने से नए
sort-smallest-to-largest = छोटे से बड़े
sort-largest-to-smallest = बड़े से छोटे
repository = रिपॉजिटरी
support = समर्थन
read-execute = पढ़ें और निष्पादित करें
deleted =
    { trash } से { $items } { $items ->
        [one] आइटम मिटाया गया
       *[other] आइटम मिटाए गए
    }
dismiss = संदेश खारिज करें
favorite-path-error = निर्देशिका खोलने में त्रुटि
progress = { $percent }%
related-apps = संबंधित ऐप्स
removing-from-recents =
    { recents } से { $items } { $items ->
        [one] आइटम हटाया जा रहा है
       *[other] आइटम हटाए जा रहे हैं
    }
remove = हटाएँ
read-write-execute = पढ़ें, लिखें और निष्पादित करें
other-apps = अन्य ऐप्स
pause = विराम
keep = रखें
permanently-deleting =
    { $items } { $items ->
        [one] आइटम स्थायी रूप से मिटाया जा रहा है
       *[other] आइटम स्थायी रूप से मिटाए जा रहे हैं
    }
read-write = पढ़ें और लिखें
none = कोई नहीं
resume = फिर से शुरू करें
grid-spacing = ग्रिड स्पेसिंग
extract-to = इस रूप में निकालें..।
delete = हटाएं
read-only = केवल पढ़ने के लिए
deleting =
    { trash } से { $items } { $items ->
        [one] आइटम मिटाया जा रहा है
       *[other] आइटम मिटाए जा रहे हैं
    } ({ $progress })..।
execute-only = केवल निष्पादित करें
details = विवरण
mount-error = ड्राइव तक पहुँचने में असमर्थ
removed-from-recents =
    { recents } से { $items } { $items ->
        [one] आइटम हटाया गया
       *[other] आइटम हटाए गए
    }
progress-paused = { $percent }%, रुका हुआ
cancelled = रद्द किया गया
operations-running-finished =
    { $running } { $running ->
        [one] ऑपरेशन
       *[other] ऑपरेशन
    } चल रहा है ({ $percent }%), { $finished } समाप्त..।
progress-failed = { $percent }%, विफल
extract-to-title = फ़ोल्डर में निकालें
open-with = इससे खोलें
permanently-deleted =
    { $items } { $items ->
        [one] आइटम स्थायी रूप से मिटाया गया
       *[other] आइटम स्थायी रूप से मिटाए गए
    }
write-execute = लिखें और निष्पादित करें
extract-password-required = पासवर्ड आवश्यक है
progress-cancelled = { $percent }%, रद्द किया गया
write-only = केवल लिखने के लिए
permanently-delete-warning = { $target } को स्थाई रूप से हटा दिया जाएगा। इस कार्रवाई को पूर्ववत नहीं किया जा सकता है।
favorite-path-error-description =
    "{ $path }" को खोला नहीं जा सकता
    "{ $path }" मौजूद नहीं हो सकता है या आपके पास इसे खोलने की अनुमति नहीं हो सकती है

    क्या आप इसे साइडबार से हटाना चाहते हैं?
empty-trash-title = रद्दी साफ़ करें?
permanently-delete-question = स्थाई रूप से हटाएं?
operations-running =
    { $running } { $running ->
        [one] ऑपरेशन
       *[other] ऑपरेशन
    } चल रहा है ({ $percent }%)..।
copy-to-title = कॉपी गंतव्य चुनें
copy-to-button-label = कॉपी
move-to-title = मूव गंतव्य चुनें
move-to-button-label = मूव
context-action = संदर्भ क्रिया
context-action-confirm-title = "{ $name }" चलाएँ?
context-action-confirm-warning =
    यह { $items } { $items ->
        [one] आइटम
       *[other] आइटम
    } पर चलेगा।
run = चलाएँ
rename-confirm = नाम बदलें
mixed = मिश्रित
pasted-image = चिपकाई गई छवि
pasted-text = चिपकाया गया पाठ
pasted-video = चिपकाया गया वीडियो
