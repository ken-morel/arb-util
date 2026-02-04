# arb-util

When you see this: `rough` it means *rough on the edges*, a leading number indicates an index
or relative roughliness, on 5.

And that's it, my second rust project. arb-util is a tool I've already worked on in julia
https://github.com/recordbreakersorg/arb-util . But here is it reimplemented, faster,
more adaptible, well... *better*.

Arb util is meant to help flutter developers like my friend https://github.com/dct-berinyuy (guess
I'm not that really into flutter myself), by handling arb files for them, including extraction
of marked strings from the file(very rough on the edges), and translating to other languages with
a local model, since those google and copilot api things are quite that expensive(seriously,
up to 6k per month!!!).

to run just get to your project root and run it:

```bash
arb-util
```

and there is an install script at repository root which build's and installs it to `/usr/bin/arb-util`.

Just in case I did not say it before, arb-util is a tool with no subcommand(since it does just the
same thing) it is devided into three files, and 3 parallel jobs.

## The extractor

Extract's marked ui strings from the dart files.
Located at [./src/extractor.rs](./src/extractor.rs)

It's role is:
- Extraction of marked strings from dart files(`rough5`). // starting not good :smiley:
- Replacing the strings in the dart files with call to `AppLocalizations`, and importing
  of the latter in case it's not already done(`rough3`).
- Adding this new strings to the main arb file in the l10n dir(both the
  directory and main arb file are read from `l10n.yaml`).

## The syncer

Synchronises the template arb file with the other ones.
Located at [./src/syncer.rs](./src/syncer.rs)

i.e when a key is added or changed in the main arb file, it add's
the key to the other files with a leading '#' indiacting it's still to be
translated, then it calls `flutter gen-l10n` to update the generated files.


## The translator

Translates the arb file strings to other languages.
Located at [./src/translator.rs](./src/translator.rs)

This assumes the existense of a `translate` script(see https://gist.github.com/4ac4e37ac61898a32b9142fcbb80c35b)
which should be a simple script taking text to translate via stdin, target language as first
argument(i.e the suffix in your arb file names, e.g en, fr, en-us) and outputs the translation.

It will run over the other arb files to look for untranslated strings(preceded with "#") and
translate them sequentially.


In future...:
-  maybe make translation parallel.. It's not so hard, just add rayon and replace the `into_iter` to `into_par_iter`.
  but that WILL surely lead to race conditions in case of several translations in the same file.
- Adding some config options for:
  - choosing the name of the translate script.
  - choosing another prefix than '#'
  (pretty easy to do by altering the source code).

## Very important advice

Don't forget to stage, and commit. Well, there are still changes that `arb-util` messes up with your
files, though that shouldn't happen.

