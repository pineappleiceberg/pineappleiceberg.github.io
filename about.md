---
layout: article
titles:
  # @start locale config
  en      : &EN       About
  en-GB   : *EN
  en-US   : *EN
  en-CA   : *EN
  en-AU   : *EN
  zh-Hans : &ZH_HANS  关于
  zh      : *ZH_HANS
  zh-CN   : *ZH_HANS
  zh-SG   : *ZH_HANS
  zh-Hant : &ZH_HANT  關於
  zh-TW   : *ZH_HANT
  zh-HK   : *ZH_HANT
  ko      : &KO       소개
  ko-KR   : *KO
  fr      : &FR       À propos
  fr-BE   : *FR
  fr-CA   : *FR
  fr-CH   : *FR
  fr-FR   : *FR
  fr-LU   : *FR
  # @end locale config
key: page-about
---

I'm Amrish Hallberg. I'm interested in the intersection of cryptography, privacy-preserving computation, and secure systems engineering. I especially like the gap between a protocol on paper and an implementation that must actually work.

I use this site as a working notebook: code-level walkthroughs, experiments, results, and the decisions behind them. Alongside classical cryptography and low-level C, I'm exploring quantum cryptography and the classical verification of quantum computation.

## Start reading

- [Bare-metal C on the Raspberry Pi Zero 2 W](/2026/10/15/blog-1.html) starts with boot and linker details on an AArch64 target.
- [Bare-metal multicore on the Pi Zero 2 W](/2026/11/06/bare-metal-multicore-pi-zero-2-w.html) follows with per-core stacks and shared state.
- [Private set intersection in C and WebAssembly](/2025/11/28/psiportable.html) compares two approaches to computing an intersection.

See the [projects page](/projects/) for a shorter overview, or explore my [public code on GitHub](https://github.com/pineappleiceberg).

## Why I write

I find it useful to show the intermediate work, including limitations and experiments that don't behave as expected. Wherever practical, I link implementations and explain the assumptions so readers can reproduce or challenge the result.

<details>
<summary>And yes, the suspicious monkey is still here.</summary>
<p>Apparently you clicked the About page. Welcome anyway.</p>
<img src="/assets/about/side-eye-monkey.gif" alt="Monkey puppet giving a suspicious side-eye" width="300" style="max-width: 100%; height: auto;">
</details>
