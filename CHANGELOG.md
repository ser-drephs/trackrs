## [3.0.0](https://github.com/ser-drephs/trackrs/compare/v2.0.0...v3.0.0) (2026-10-24)

### ⚠ BREAKING CHANGES

* use `start` instead of `continue` and `break` instead of `disconnect`.

### Features

* add `add` command ([72bd10c](https://github.com/ser-drephs/trackrs/commit/72bd10c190c4886db2559802c4b0c6cd039838b6))
* add `sick` and `vacation` subcommands ([cd1126e](https://github.com/ser-drephs/trackrs/commit/cd1126ee8330070a7e08234da7d5aa1381ef0f1d))
* add cumulated column to status table ([fc78a27](https://github.com/ser-drephs/trackrs/commit/fc78a27d7f0d9642aa8bf7a45a8c60ca8f519d93))
* add day status to show the status of another day ([a5bd047](https://github.com/ser-drephs/trackrs/commit/a5bd04789e1956fc57afbb385d61974e869be831))
* add monthly status ([8f1b51a](https://github.com/ser-drephs/trackrs/commit/8f1b51ac6b3bef7f94b05d7d0ded0da722fcf02e))
* add notes to entry ([ba1e283](https://github.com/ser-drephs/trackrs/commit/ba1e2836919535b028d015a268b13e7886b05b7f))
* add overtime generation and consideration for status ([e532a20](https://github.com/ser-drephs/trackrs/commit/e532a20ce7cb1e2f23b3f8f7dab770fdd25666cd))
* add year status ([e5bcf2d](https://github.com/ser-drephs/trackrs/commit/e5bcf2d192d3ac76710ac7efecd610e75bed2dec))
* use table layout for week by default ([a7d6889](https://github.com/ser-drephs/trackrs/commit/a7d68893c45cd4dde58736fafa8081efbdfc453e))

### Bug Fixes

* command `takeover` and fill break calculation ([dbb6270](https://github.com/ser-drephs/trackrs/commit/dbb6270716d6e1822d16860177412bede006b66a))
* empty weekly status entries show overall overtime ([ff8d318](https://github.com/ser-drephs/trackrs/commit/ff8d3188f40fb403783ccf8f3ba80f363183d6c0))
* keep time files valid after editing them by hand ([c1377c8](https://github.com/ser-drephs/trackrs/commit/c1377c88d23c4bd599da002665e912de93cc0f95))

### Code Refactoring

* remove `continue` and `disconnect` subcommands ([2e34a05](https://github.com/ser-drephs/trackrs/commit/2e34a0513f65168a85e0129285555abcccc2e920))

## [2.0.0](https://github.com/ser-drephs/trackrs/compare/v1.4.1...v2.0.0) (2025-03-25)


### ⚠ BREAKING CHANGES

* add version to model
* use utc time

### Features

* add version to model ([7d60167](https://github.com/ser-drephs/trackrs/commit/7d60167936189a53f56551c193acd55e9bf0811e))


### Code Refactoring

* use utc time ([eb36d5d](https://github.com/ser-drephs/trackrs/commit/eb36d5dde09cb1b3d3f5cb9abe360a35c85ca807))

### [1.4.1](https://github.com/ser-drephs/trackrs/compare/v1.4.0...v1.4.1) (2023-05-19)


### Bug Fixes

* status table under linux ([fdd2693](https://github.com/ser-drephs/trackrs/commit/fdd269321ffe8bfad58106ca31cbf7ffc4115ec1))

## [1.4.0](https://github.com/ser-drephs/trackrs/compare/v1.3.0...v1.4.0) (2023-02-19)


### Features

* format week list as table ([a9abb3f](https://github.com/ser-drephs/trackrs/commit/a9abb3fe37847db6b16bea99d2afa8d7fcc54e48))
* week status crossing year ([4c99acd](https://github.com/ser-drephs/trackrs/commit/4c99acd9658c14d477354caff5e9c926a4e0ce97))

## [1.3.0](https://github.com/ser-drephs/trackrs/compare/v1.2.0...v1.3.0) (2022-09-09)


### Features

* limits in minutes ([f83150c](https://github.com/ser-drephs/trackrs/commit/f83150c18da907feb29cc8c7a644e3959b56d548))

## [1.2.0](https://github.com/ser-drephs/trackrs/compare/v1.1.0...v1.2.0) (2022-09-07)


### Features

* work time per day in minutes ([17f672c](https://github.com/ser-drephs/trackrs/commit/17f672c5fd866a23f7efb14011c72d0ad0a80368))


### Bug Fixes

* status showing wrong values with temp end ([308789f](https://github.com/ser-drephs/trackrs/commit/308789fbe458507d9532ca03191bf6257eeb3f84))
* takeover cli ([964ef20](https://github.com/ser-drephs/trackrs/commit/964ef207ba5314edadeebee78f68185b226eab96))
* takeover folder may not be available ([b6f8f48](https://github.com/ser-drephs/trackrs/commit/b6f8f48f90ec43760fd38169ba0371b4cecdab03))

## [1.1.0](https://github.com/ser-drephs/trackrs/compare/v1.0.1...v1.1.0) (2022-09-04)


### Features

* takeover time to next day ([2e1e6b9](https://github.com/ser-drephs/trackrs/commit/2e1e6b9286cf635a39a2a8a13d367a9cfc0dbaa4))

### [1.0.1](https://github.com/ser-drephs/trackrs/compare/v1.0.0...v1.0.1) (2022-09-04)


### Bug Fixes

* assert break creates entries before end ([076ab12](https://github.com/ser-drephs/trackrs/commit/076ab1209df0d22a8f7123500b2d2534695e1f16))
* end status without break panic ([b03141b](https://github.com/ser-drephs/trackrs/commit/b03141bf45e1e7a7079a2ef76c4099db51cfff71))
* expected break not correctly shown in status ([b7deb36](https://github.com/ser-drephs/trackrs/commit/b7deb36d83f1690cefd03f18ddb6d47b7c76e7b0))

## 1.0.0 (2022-08-30)


### Features

* add initial cli ([b9ef7ce](https://github.com/ser-drephs/trackrs/commit/b9ef7cee842dc62d27195994e545604cb059ce45))
