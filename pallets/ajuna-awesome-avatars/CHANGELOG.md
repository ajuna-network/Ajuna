
# Change Log
All notable changes to the Awesome Ajuna Avatars will be documented in this file.
 
 ## [0.1.17] - 2023-04-13
  
Fixed Treasury claim, reclaimed burned balances to total issuance (sudo).

### Fixed
- Add mutate extrinsic
  Fixed dna swap for old mapping of body color.

## [0.1.14] - 2023-03-01
  
Relase following the Season 1 ending, addressing a forging fix and adding the transfer feature.
 
### Added

- [PLAT-752] Transfer avatar
  Avatars are now transferable between accounts, there is a transfer fee set to 1 BAJU.
 
### Changed
- [PLAT-743] Add extrinsics to manage free mints
  Free mints can now be set to a certain account by the organizier of the season.
- [PLAT-754] Remove unused free mint calls
  There set free mints make issue and withdraw useless, so they are removed.
 
### Fixed
- [PLAT-745] Roll for each sacrifice 
  According to the game logic each sacrificed avatar allows for one roll, this was not the case for Avatars with more sacrifices then upgradeable components.
 
## [0.1.10] - 2023-02-14
 Initial season 1 Awesome Ajuna Avatars!
 
### Added
  None
   
### Changed
  None
 
### Fixed
  None
