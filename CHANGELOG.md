# Changelog

## [0.1.3](https://github.com/pmqueiroz/mail-sieve/compare/v0.1.2...v0.1.3) (2026-10-31)


### Features

* **server:** read the avatar from community plugin manifests ([32ddaf7](https://github.com/pmqueiroz/mail-sieve/commit/32ddaf7fbbd14ecc1ec36cb160a0baf5f9e7ff26))
* **web:** animate the ghoul shrinking when it naps ([d3deda7](https://github.com/pmqueiroz/mail-sieve/commit/d3deda7ba6f996bd921244cbf1511f826015584e))
* **web:** drop the core plugins section note ([ca89bde](https://github.com/pmqueiroz/mail-sieve/commit/ca89bde7a3e4b14d219f6c5a679e2c427e7ae703))
* **web:** drop the wingman napping tip ([58ac396](https://github.com/pmqueiroz/mail-sieve/commit/58ac39602ef61501939fd19f194356a0e881856a))
* **web:** scale buttons down while pressed ([f744d8d](https://github.com/pmqueiroz/mail-sieve/commit/f744d8d371c18d218525ecca4d28f173ed491624))
* **web:** show an icon and color avatar on plugin cards ([fd98dfe](https://github.com/pmqueiroz/mail-sieve/commit/fd98dfeba15d71de9b9a85296eabc1084a2d54dc))
* **web:** slide the segmented control thumb between options ([6e1d9c5](https://github.com/pmqueiroz/mail-sieve/commit/6e1d9c5659c0a8ba91df956ac31390b563591b0c))

## [0.1.2](https://github.com/pmqueiroz/mail-sieve/compare/v0.1.1...v0.1.2) (2026-10-31)


### Features

* **cli:** add rules export and import commands ([688b10e](https://github.com/pmqueiroz/mail-sieve/commit/688b10ed4d385a6e88f47645a33e028c094d8e78))
* **core:** parse domain blocklists in hosts and adblock formats ([6e50bba](https://github.com/pmqueiroz/mail-sieve/commit/6e50bba470269c8cf7c3042c6b269eeea282102f))
* **server:** add an ai completion endpoint for plugins ([1808ab0](https://github.com/pmqueiroz/mail-sieve/commit/1808ab0ace8d7006d5d024017dbdd55f9964f485))
* **server:** add an api to pin and unpin senders and domains ([32e9e35](https://github.com/pmqueiroz/mail-sieve/commit/32e9e35d38289c796e548b0648e3f953277cddd7))
* **server:** add plugin state, config and ai model endpoints ([91ac404](https://github.com/pmqueiroz/mail-sieve/commit/91ac404272a4a9a8a3eedaf8ca52cf1757d296ca))
* **server:** export and import rules as yaml ([0606146](https://github.com/pmqueiroz/mail-sieve/commit/060614611b0aac9c333f060055cde25dc5b0329e))
* **server:** list senders that keep mailing after an unsubscribe ([b890a2b](https://github.com/pmqueiroz/mail-sieve/commit/b890a2b68c77adcb0e1bf72e8c22287cf0d8fb31))
* **server:** preview which blocklisted domains sent you mail ([e26b3aa](https://github.com/pmqueiroz/mail-sieve/commit/e26b3aa2777d2be80d5afb7b45e5b3315d24e797))
* **server:** skip pinned senders in bulk actions and rules ([e753679](https://github.com/pmqueiroz/mail-sieve/commit/e7536794cbefb886c8b447e38baf5cb41b01515d))
* **site:** add the github sponsor heart to sponsor buttons ([4f65b1d](https://github.com/pmqueiroz/mail-sieve/commit/4f65b1ddd3e79adff32056a9925d5862e58c2647))
* **store:** list the rules of one account ([74165c6](https://github.com/pmqueiroz/mail-sieve/commit/74165c67abf4e9985a9ff0ff2a0ee89b62da998c))
* **store:** pin senders and domains with an is:pinned filter ([59e91e4](https://github.com/pmqueiroz/mail-sieve/commit/59e91e4efbd9dfd800e4f2645d2c98c27a62103b))
* **store:** report the covering pin on message rows ([670e04f](https://github.com/pmqueiroz/mail-sieve/commit/670e04f3b1348088b2981b7dd3a075728fcb22b7))
* **unsub:** add a guarded fetch for public text files ([e525fad](https://github.com/pmqueiroz/mail-sieve/commit/e525fad76abd5ddb01ebacc8ea4d2ef9e3686fe8))
* **web:** add plugin host and plugins page ([a1702c5](https://github.com/pmqueiroz/mail-sieve/commit/a1702c55499f7ae9b7edf805b16524e1448b0e1f))
* **web:** add the ai categorizer core plugin ([799a280](https://github.com/pmqueiroz/mail-sieve/commit/799a280a99651deed1e00aeaadd75957cfda8804))
* **web:** add the ghoul wingman core plugin ([7c4c885](https://github.com/pmqueiroz/mail-sieve/commit/7c4c8850beec9c9b2a37e022a94d01eb6b64828d))
* **web:** add the query builder core plugin ([e633e87](https://github.com/pmqueiroz/mail-sieve/commit/e633e8759df892451b2e90bfacd78017c1bf2ccd))
* **web:** add the rule recipes core plugin ([57b04fe](https://github.com/pmqueiroz/mail-sieve/commit/57b04fe248620852e67be002656d0a46cdc88c5a))
* **web:** add the shared blocklists core plugin ([dce7477](https://github.com/pmqueiroz/mail-sieve/commit/dce74777a93d1ae9f3a10eba689993d1c924b10e))
* **web:** add the undo history core plugin ([144f2ed](https://github.com/pmqueiroz/mail-sieve/commit/144f2ed407b9d792983c7bbf54184a11589b9594))
* **web:** add the unsubscribe audit core plugin ([70dbdd7](https://github.com/pmqueiroz/mail-sieve/commit/70dbdd78daf7bcc0618f0ec98c34ac2bf1388644))
* **web:** add the weekly digest core plugin ([b651b7a](https://github.com/pmqueiroz/mail-sieve/commit/b651b7a641b2359991900ff65ee98bfe35f4802b))
* **web:** add topbar, overview, rules, search and activity plugin slots ([1d1a8c8](https://github.com/pmqueiroz/mail-sieve/commit/1d1a8c84edb4b4772e835761fd165fab61b0b621))
* **web:** export and import rules from the rules screen ([28d7bdc](https://github.com/pmqueiroz/mail-sieve/commit/28d7bdc6d5eac9a814029cf203d958d9b77962a1))
* **web:** expose ai completions on the plugin host ([4e901da](https://github.com/pmqueiroz/mail-sieve/commit/4e901da0af6e4e2bd85caac3a776bd2241a99cae))
* **web:** let the wingman pick suggestions with an ai model ([507c108](https://github.com/pmqueiroz/mail-sieve/commit/507c10895f16fdec9c02276a103ff6bed5a28edf))
* **web:** note when an unsubscribe follow-up skips a pinned sender ([a98c2a7](https://github.com/pmqueiroz/mail-sieve/commit/a98c2a7e4cfe779a076c25b4004c47eee3b7d84c))
* **web:** pin senders and domains from the senders view ([5bac090](https://github.com/pmqueiroz/mail-sieve/commit/5bac0907ecdf55ce03f3ee95a60a63d0c19f2560))
* **web:** pin senders from the message preview ([acab784](https://github.com/pmqueiroz/mail-sieve/commit/acab784a925c378af37f9491dad61f10f283c4c1))
* **web:** present the ghoul wingman as an ai mail assistant ([3269b4c](https://github.com/pmqueiroz/mail-sieve/commit/3269b4ccc87a6e87a3dbd8af18c6f17496c52f34))
* **web:** show pinned messages a rule will skip ([574277a](https://github.com/pmqueiroz/mail-sieve/commit/574277a879f2a75148d548ee96baf7e20fa448f8))
* **web:** turn every core plugin off by default ([4f37ac3](https://github.com/pmqueiroz/mail-sieve/commit/4f37ac3d2c996670bed4c4fa7e21f7f6bd3fcfd9))


### Bug Fixes

* **web:** stretch the blocklist url field across its row ([d880493](https://github.com/pmqueiroz/mail-sieve/commit/d880493bc6f004d4e2898f810ec01af55b1a61d2))

## [0.1.1](https://github.com/pmqueiroz/mail-sieve/compare/v0.1.0...v0.1.1) (2026-10-31)


### Features

* **site:** add png favicons, apple-touch icon and web manifest ([fe2a9c0](https://github.com/pmqueiroz/mail-sieve/commit/fe2a9c037f244723708a325b1e3bce6eed60ad63))
* **site:** brand the site copy as mailsieve ([4b8e965](https://github.com/pmqueiroz/mail-sieve/commit/4b8e965c45b8d773c61ecef1f699254f7e3b2aa4))
* **site:** drop the eyebrow labels and hero tagline ([1ecf415](https://github.com/pmqueiroz/mail-sieve/commit/1ecf4150e8620e938dfa739a4b25be7455d8290f))
* **site:** hide the hero screenshot caption visually ([455dc89](https://github.com/pmqueiroz/mail-sieve/commit/455dc893bf68a85f33d4d34f36a81e65f7b9b9bc))
* **site:** remove the license link from the footer ([372cccd](https://github.com/pmqueiroz/mail-sieve/commit/372cccd9168448ce0f5bfaddd0c10e2f4aa8bea2))
* **site:** render the product name as the mailsieve wordmark ([df1cd27](https://github.com/pmqueiroz/mail-sieve/commit/df1cd27f8699672d6fb19bddf40d56fa2c51f4ac))
* **site:** replace the open graph image ([9fb8349](https://github.com/pmqueiroz/mail-sieve/commit/9fb8349acb467d30c5b9481c83e7711c0700172d))
* **site:** strip the status and run details from the rule panel ([dd605d5](https://github.com/pmqueiroz/mail-sieve/commit/dd605d5fe391c63081a48966a819ef9274c95d12))
* **web:** apply the new favicon set ([bfa440d](https://github.com/pmqueiroz/mail-sieve/commit/bfa440ddd6c7cde9d695566a8526ed0115f35d48))
* **web:** render the product name as the mailsieve wordmark ([1bcfbc5](https://github.com/pmqueiroz/mail-sieve/commit/1bcfbc5f2856a1066aa985dd041b40110c4d5c7d))
