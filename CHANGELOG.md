# Changelog

<!-- START TEMPLATE

## ✨ **VERSION** *(DATE)*

- #### ⚡️ Features
  - Sample
- #### 🛠 Fixes
  - Sample
- #### 🚨 Breaking changes
  - Sample

END TEMPLATE-->

## ✨ **0.8.0** *(TBD)*

## ✨ **0.7.0** *(2019-11-11)*

- #### ⚡️ Features
  - Greatly improve the quality of matcher string parsing errors. [https://github.com/yewstack/yew_router/issues/149]
  - Bring back `{}`, `{*}`, and `{<number>}` capture syntax for tuple structs/enums variants. 
  If your variant or struct doesn't have named fields, you don't need to supply names in the matcher string [https://github.com/yewstack/yew_router/issues/116]
  - Redirects that happen in the `Router` component actually change the url in the browser [https://github.com/yewstack/yew_router/issues/171]
  - Allow parsing (almost) any character after a `#` is encountered in matcher strings. 
  This enables this library to be used as a fragment router. [https://github.com/yewstack/yew_router/issues/150]
- #### 🛠 Fixes
  - Allow `!` to appear after `{...}` in matcher strings. [https://github.com/yewstack/yew_router/issues/148]
  - Matcher strings can now start with `&`. [https://github.com/yewstack/yew_router/issues/168] 
- #### 🚨 Breaking changes
  - Upgrade to Yew 0.10.1 
  - Switch components now need to implement `Clone` in order to be used with the `Router` [https://github.com/yewstack/yew_router/issues/171]


## ✨ **0.7.0** *(2019-11-11)*
- #### ⚡️ Features
  - `Switch` trait and Proc Macro enables extracting data from route strings.
  - `Router` component added.
  - `RouterLink` and `RouterButton` helper components added.
- #### 🚨 Breaking changes
  - Nearly everything. Most items were renamed.
  - Upgrade to Yew 0.9.0
