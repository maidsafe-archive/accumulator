# Accumulator

[![](https://img.shields.io/badge/Project%20SAFE-Approved-green.svg)](http://maidsafe.net/applications) [![](https://img.shields.io/badge/License-GPL3-green.svg)](https://github.com/maidsafe/accumulator/blob/master/COPYING)

**Primary Maintainer:** Fraser Hutchison (fraser.hutchison@maidsafe.net)

|Crate|Linux/OS X|Windows|Coverage|Issues|
|:---:|:--------:|:-----:|:------:|:----:|
|[![](http://meritbadge.herokuapp.com/accumulator)](https://crates.io/crates/accumulator)|[![Build Status](https://travis-ci.org/maidsafe/accumulator.svg?branch=master)](https://travis-ci.org/maidsafe/accumulator)|[![Build status](https://ci.appveyor.com/api/projects/status/1imtexgsshnpxnvn/branch/master?svg=true)](https://ci.appveyor.com/project/MaidSafe-QA/accumulator/branch/master)|[![Coverage Status](https://coveralls.io/repos/maidsafe/accumulator/badge.svg)](https://coveralls.io/r/maidsafe/accumulator)|[![Stories in Ready](https://badge.waffle.io/maidsafe/accumulator.png?label=ready&title=Ready)](https://waffle.io/maidsafe/accumulator)|

| [API Documentation - master branch](http://maidsafe.net/accumulator/master) | [SAFE Network System Documentation](http://systemdocs.maidsafe.net) | [MaidSafe website](http://maidsafe.net) | [SAFE Network Forum](https://forum.safenetwork.io) |
|:------:|:-------:|:-------:|:-------:|

## Overview

A key-value store limited by size or time, allowing accumulation of multiple values under a single key.

## Todo Items

### [0.2.0]
- [ ] Add time point to allow removal of old items (save memory leak)
- [ ] API version 1.0.0
