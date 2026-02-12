# arb-util

When you see this: `rough` it means *rough on the edges*, a leading number indicates an index
or relative roughliness, on 5.

And that's it, my second rust project. arb-util is a tool I've already worked on in julia
https://github.com/recordbreakersorg/arb-util . But here is it reimplemented, faster,
more adaptible, well... *better*.

Arb util is meant to help flutter developers like my friend https://github.com/dct-berinyuy (guess
I'm not that really into flutter myself), by handling arb files for them, including extraction
of marked strings from the file(very rough on the edges), and translating to other languages with
gemini api using `gemini-2.5-flash-lite` which has flash-speed, and is free as of now at least,
since most chatgpt and copilot api things are quite that expensive(seriously,
up to 6k per month!!! For work yeah but not for translations).

## usage

to run just get to your project root and run it:

```bash
arb-util
```

and there is an install script at repository root which build's and installs it to `/usr/bin/arb-util`.

Then to mark strings to be extracted, preceed them with a `_` and save, arb-util should replace
them fast enough, so if you're on an editor like helix which does not reload files modified by external
processes you should reload pretty must just after.

You can still edit the main arb file(which will invalidate translations in the other ones), or
alter the translations in an other arb file, but preferably kill `arb-util` before doing that,
to help you this's the command: `pkill -9 arb-util`, though `pkill -INT arb-util` would be nicer,
have a good time!

Just in case I did not say it before, arb-util is a tool with no subcommand(since it does just the
same thing) it is devided into three asynchronious tasks, for yes I delegated most of it's work to
tasks, and hope the scheduler is a good one.

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

It uses google ai's gemini api to query `gemini-2.5-flash-lite`, I'm sure I said this already, because
it works pretty well, should not consume that much and is ultra-fast.
It will run over the other arb files to look for untranslated strings(preceded with "#") and
translate them in parallel, or I should precise the requests are done in seperate tasks(with of course
a single writter task).

## Gemini api

arb-util reads `GEMINI_API_KEY` environment variable and makes a `reqwest` at the openai compatible
url.

## Very important advice

Don't forget to stage, and commit. Well, there are still changes that `arb-util` messes up with your
files, though that shouldn't happen, I use it myself in [Book Bridge](https://github.com/ken-morel/book-bridge).
