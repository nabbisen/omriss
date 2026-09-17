# Languages

The GUI ships with **English** and **日本語 (Japanese)** catalogs and can be
switched at runtime from the toolbar. Missing translations fall back to
English, so the interface never shows an empty label.

**On startup**, omriss picks up your OS's preferred UI language automatically
if it's one of the ones above — Windows, Linux, and macOS alike — and falls
back to English otherwise. There is no separate language setting to configure
beforehand; switching languages from the toolbar only changes what's shown
for the rest of that session; the automatic startup detection runs again the
next time you launch omriss.

Your *documents* are never translated or transformed: language settings only
affect the GUI chrome.
