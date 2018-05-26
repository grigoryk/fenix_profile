# Fenix Profile experiments
## Mentat
This work stemmed out of a [mentat](https://github.com/mozilla/mentat) issue: https://github.com/mozilla/mentat/issues/698, which provides some context.

To quote the original ticket:
```
This is pretty open ended. We need to:

- model places and history in Mentat
- store history URLs and history visit data
- query history efficiently, weighting by frecency-like measures
- produce top sites-like materialized views (similar to Firefox for iOS)
- allow to browse history data, like in the library views in Desktop or the home panels in Firefox for Android

This is really about figuring out what functionality we need to support for our browsers, starting with a drop-in for Firefox for Android.

This isn't really a Mentat issue, but I think it makes sense to experiment very close to Mentat while we learn. There's a bunch of different directions this could go: deeply into the product experience, deeply into the details of a future sync, deeply into the performance profile.
```

## Fenix?
I'll be figuring out how to glue this into [Fenix Browser](https://github.com/mozilla-mobile/fenix/) as its storage backend, to act as an initial, realistic test vehicle.