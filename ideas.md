# Main Idea

Vampire Survivors like game with an emphasis on players collecting notes when they level up and creating a score for instruments to play

- The player can move (slowly) in real-time
- Instruments and enemies do their abilities on the beat
- The player has to hit the correct keyboard key corresponding to the notes that they put on the score for the instrument to actually play that note
- Enemies drop xp when they are killed that needs to be collected
- Fixed map to avoid level creation and camera controlling complexity

## On Level Up pick from 3 choices of notes with these features

1. Note Length

- eighth note, quarter note, etc. determines how long the instrument's ability lasts
- rarity for longer notes

2. Dynamic

- ppp, pp, mp, mf, f, ff, etc to determine the power of the ability
- rarity for more powerful dynamics

## Instrument Abilities

1. Laser Beam shoots in a straight line and can hit multiple targets

- Pulses for damage on each beat
- Note length determines how long the laser is active
- Each dynamic level can add to Range, Width, Damage per pulse
- Does base damage

2. AoE ring around the instrument

- Pulses for damage on each beat
- Length of note determines how long the AoE is active
- Each dynamic level can add to Range, Damage per pulse
- Negative damage multiplier

3. Bullets hit single targets

- Fires bullet(s) on each beat it is active
- Dynamic level can add to number of bullets fired per beat, damage per bullet
- Positive dmg multiplier

## Questions

- Enemy variety?
- How do we get new instruments?
  - Buy them with a secondary currency?
  - Offered every X levels?
  - Buy in between levels?
- Instrument movement?
  - Follows the player?
  - Fixed spots on the map that you unclock with a secondary currency?
- Can you have multiple of the same instrument?
- Player sprint on the beat by hitting a key?
- Change where a note is on the score on the fly?
- Slam notes together to make them more powerful?
- What's the game loop?
  - Beat a boss
  - Score to beat the level
  - Survive for a certain amount of time
- Carry instruments over levels, but not notes?
- Is there meta progression?

## Stretch ideas

### Support instrument types

1. Buff solo instruments

- Length of note determines how long the AoE is active
- Dynamic level can add to AoE size, increase beat rate (pulses every 16th instead of every 8th), increase damage per pulse

2.  Slow enemies

- Length of note determines how long the AoE is active
- Dynamic level can add to AoE size, length of effect, style of effect slow -> freeze

### Elemental notes (maybe can use note values? a, b, c, d, e, f, g)

1. Poison - DoT damage

- Laser only?
- Goes through armor
- Add to a support instrument to prolliferate the poison, make the DoT instantly do it's damage

2. Fire

- Bullet only?
- Adds AoE explosion
- Support instrument to make the explosions bigger

3. Water

- AoE only?
- Pushes enemies away
- Add to a support instrument to make the push do damage for distance the enemy is pushed

### Easy mode

- Notes play automatically instead of needing to hit the key
- More powerful if you do manage to hit the key

### Score upgrades

- Repeat a measure without having to play it with repeat

  <img width="236" height="150" alt="image" src="https://github.com/user-attachments/assets/72bdd80c-a05c-42d7-8067-404ef57179ae" />


### Defenses for the player

- Speed increases
- Health increases
- Sprint cooldown decreases
