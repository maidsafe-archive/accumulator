# Accumulator

**Maintainer:** Andreas Fackler (andreas.fackler@maidsafe.net)

|Crate|Linux/OS X|Windows|Coverage|Issues|
|:---:|:--------:|:-----:|:------:|:----:|
|[![](http://meritbadge.herokuapp.com/accumulator)](https://crates.io/crates/accumulator)|[![Build Status](https://travis-ci.org/maidsafe/accumulator.svg?branch=master)](https://travis-ci.org/maidsafe/accumulator)|[![Build status](https://ci.appveyor.com/api/projects/status/1imtexgsshnpxnvn/branch/master?svg=true)](https://ci.appveyor.com/project/MaidSafe-QA/accumulator/branch/master)|[![Coverage Status](https://coveralls.io/repos/maidsafe/accumulator/badge.svg)](https://coveralls.io/r/maidsafe/accumulator)|[![Stories in Ready](https://badge.waffle.io/maidsafe/accumulator.png?label=ready&title=Ready)](https://waffle.io/maidsafe/accumulator)|

| [API Documentation - master branch](http://docs.maidsafe.net/accumulator/master) | [SAFE Network System Documentation](http://systemdocs.maidsafe.net) | [MaidSafe website](http://maidsafe.net) | [SAFE Network Forum](https://forum.safenetwork.io) |
|:------:|:-------:|:-------:|:-------:|

## Overview

A key-value store limited by size or time, allowing accumulation of multiple values under a single key.

## Todo Items

### [0.2.0]
- [ ] Add time point to allow removal of old items (save memory leak)
- [ ] API version 1.0.0

## License

Licensed under either of

* the MaidSafe.net Commercial License, version 1.0 or later ([LICENSE](LICENSE))
* the General Public License (GPL), version 3 ([COPYING](COPYING) or http://www.gnu.org/licenses/gpl-3.0.en.html)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the MaidSafe Contributor Agreement, version 1.1 ([CONTRIBUTOR]
(CONTRIBUTOR)), shall be dual licensed as above, and you agree to be bound by the terms of the
MaidSafe Contributor Agreement, version 1.1.
