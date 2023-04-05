# Changelog

with help from [`git log`](https://www.git-scm.com/docs/git-log):

```shell
git log --oneline --format="- [%h](https://github.com/villagekit/robokit/commit/%H): %s"
```

## [robokit-0.2.0](https://github.com/villagekit/robokit/releases/tag/robokit-0.2.0)

- [a4832dc](https://github.com/villagekit/robokit/commit/a4832dc78d6c03d637b7fb0c4584d36336d31679): use explicit heapless sizes (#33)
- [55c3263](https://github.com/villagekit/robokit/commit/55c3263137d6ce35e39c21e3301d5d2a75b43e50): super time 2 (#32)

## [robokit-0.1.0](https://github.com/villagekit/robokit/releases/tag/robokit-0.1.0)

- [d4a7c2c](https://github.com/villagekit/robokit/commit/d4a7c2c73e912032651dbdcbd2db395f3e92ccd2): too many keywords, sorry crates.io
- [9c74370](https://github.com/villagekit/robokit/commit/9c743701ffdde27651eaac5dd62d876521d2f806): step by step libraryify 3 (#31)
- [8004ceb](https://github.com/villagekit/robokit/commit/8004cebdc23ac7c7f4f6a9e419358ef2f84556a7): step by step libraryify 2 (#30)
- [67c93e7](https://github.com/villagekit/robokit/commit/67c93e74af2a7ed62e0c205d499a50b555fcbd19): step by step libraryify (#28)
- [afbaa72](https://github.com/villagekit/robokit/commit/afbaa729afb47ae7ca0a103baa34819949ebc9d7): add axis homing (#20)
- [dde54d3](https://github.com/villagekit/robokit/commit/dde54d337aa03030b8ea17d65f2896f866ea3770): generic traits (#27)
- [c645309](https://github.com/villagekit/robokit/commit/c645309cb22a25e1e9f6cb99f2cb99a92d47062d): tidy: remove rtic, upgrade crates, add heap alloc (#26)
- [e23970a](https://github.com/villagekit/robokit/commit/e23970afc631e851ddd3267ae73a265af4013a06): upgrade deps, use fugit time with stepper (#24)
- [bb424c1](https://github.com/villagekit/robokit/commit/bb424c13896f65a5ed9d83a582f8d06d22691d8d): update deps to latest versions
- [0d73356](https://github.com/villagekit/robokit/commit/0d733562c8db8be5ff5cfa3843e98d6a149ada29): use fugit time with stepper
- [037ec1c](https://github.com/villagekit/robokit/commit/037ec1c9af785bd104999f210b63c2b77a799191): super time (#25)
- [3d04058](https://github.com/villagekit/robokit/commit/3d040587fad06fe89250f9fa0ed54c429e2e204f): improve command flow (#23)
- [442c1ca](https://github.com/villagekit/robokit/commit/442c1caa515f4c2608a50cbf4ddb3646d05f8bb6): use merged `stm32f7xx-hal` version
- [e953ea8](https://github.com/villagekit/robokit/commit/e953ea86dfafb2878f1899340c51fe38c45a5d78): use published `rmodbus@0.6.1`
- [6fabab0](https://github.com/villagekit/robokit/commit/6fabab0ce2f1a47327714a78a7c5d2fa8fb40e51): iHSV57 servo spindle: part 2 (#21)
- [0e0dfb5](https://github.com/villagekit/robokit/commit/0e0dfb5e16aa395ef0e8f225346c610b7472b171): iHSV57 servo spindle (#19)
- [6390f37](https://github.com/villagekit/robokit/commit/6390f37e3d649d41216190ec022aae710223fd2a): add switch sensor (#15)
- [6b7a74d](https://github.com/villagekit/robokit/commit/6b7a74d5c294d3cfe08c439307a18c26997f5065): update docs and add license (#13)
- [3ea451e](https://github.com/villagekit/robokit/commit/3ea451ee20c6d049e157b2449d1972b2e56a9ff8): stepper axis (#12)
- [5bfe5df](https://github.com/villagekit/robokit/commit/5bfe5dfb2f5d14013f8a2f9e896ef4997572ae83): remove unused src/error.rs
- [9412bec](https://github.com/villagekit/robokit/commit/9412becce3c0930da490fb8574a9f30c3e4b7a1c): implement command system (#11)
- [e954227](https://github.com/villagekit/robokit/commit/e95422701247898d141c6a26b7e2b9048c795047): Merge pull request #10 from villagekit/rusty-revival
- [0ca89fc](https://github.com/villagekit/robokit/commit/0ca89fcf7376f23aadb1c7b22e896410db289211): follow latest knurling-rs/app-template
- [8ffbd1d](https://github.com/villagekit/robokit/commit/8ffbd1dfb69f47f709a1aba23c3d323e82581079): a fresh new beginning

## C++ prototype

- [1405407](https://github.com/villagekit/robokit/commit/140540713791b93767f0ec5e1bf4c4d4946cc533): improve stepper motor code (#6)
- [e06dc44](https://github.com/villagekit/robokit/commit/e06dc446741e124882bc86e3e64efc9675cbd1f4): interrupt safe queue (#5)
- [179f766](https://github.com/villagekit/robokit/commit/179f7667845aec531c1827cac82088bff07fb72f): remove variant.hpp (in ./3rdparty)
- [7f125fa](https://github.com/villagekit/robokit/commit/7f125facf64083ccba83933e72ba5ca0a77f11a1): add dev instructions to readme
- [1e4a697](https://github.com/villagekit/robokit/commit/1e4a69716e340628aced00d3e59924d03af1d0ea): pulse motors (#4)
- [2a91413](https://github.com/villagekit/robokit/commit/2a9141305d36a83cff89de3b0e66dbb37d7107f0): send json data to browser and render (#3)
- [3dd87a4](https://github.com/villagekit/robokit/commit/3dd87a4a9e1eebfee3566db285892fcb6f4bb36e): clean up code (#2)
- [dc049da](https://github.com/villagekit/robokit/commit/dc049dae17e1b911fbf61c4d0ecd016ab62b65bf): http server (#1)
- [fcbaa54](https://github.com/villagekit/robokit/commit/fcbaa545150bd6220d1d6466fa06484af1a1b556): now works in default (old) C++ (14?)
- [5ebdc95](https://github.com/villagekit/robokit/commit/5ebdc954e31631415d989b2b99f801304e2ea3ff): yes it works!
- [a97315e](https://github.com/villagekit/robokit/commit/a97315eb9c0bc442b90aa07ec970f6501563c0dd): attempt a new redux state approach!
- [6d86a1f](https://github.com/villagekit/robokit/commit/6d86a1fdb4aba28fd34bee79267eeeec0055f526): a boring class approach
- [131d447](https://github.com/villagekit/robokit/commit/131d447338658ceb95fa5635f2850eecdb9e4f4e): nope
- [d7bc052](https://github.com/villagekit/robokit/commit/d7bc052404a4630cacc7c08ab5d662a71135ce99): get fancy c++17 std::variant state machines working
- [0e00c81](https://github.com/villagekit/robokit/commit/0e00c81bed20d2400c0024a58c8d87e2148ac135): failed attempt using jrullan/StateMachine
- [9e21bf7](https://github.com/villagekit/robokit/commit/9e21bf767d5f6da3e2df3596074516fbf437a2e5): naive blink machines
- [d9f0462](https://github.com/villagekit/robokit/commit/d9f046288d4c3a329bb277de9f2bb8cadc78ebcd): blink with stm32 timer interrupts
- [e8897c0](https://github.com/villagekit/robokit/commit/e8897c0f8fb1f715e73776255d3307e4cb66ebc1): in the beginning
