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
- I'll be figuring out how to glue this into something like [Fenix Browser](https://github.com/mozilla-mobile/fenix/) as its storage backend, to act as an initial, realistic test vehicle.
- The goal is to use this from within a mobile app - Android or iOS.

## Why not use mentat directly?
A lot of the functionality around "profiles" could be - and possibly should be - achieved by interacting directly with mentat via its FFI interface. Examples of that are:
- defining and evolving schema
- basic querying
- transacting

"History" provides an interesting challenge in that its not simply CRUD data - we would like to perform some non-trivial aggregations and querying in order to power something like awesomebar or Top Sites of a browser.

This little project will explore approaches to sharing as much of that work as possible in a common Rust layer, and exposing a sensible API to various mobile consumers (Android and iOS as primary targets).

Additionally, we'd like our awesomebar queries and Top Sites to be fast, which likely means caching, likely at a level beyond what mentat intends to support. This implies a caching indirection at the library level.