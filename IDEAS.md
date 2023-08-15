
# Hewo, ideas for the game

## Concept and theme

A frog dreams, and the dream consists of a sequence of battles using deck-building mechanics. It should be a roguelike deckbuilder, so no permanent upgrades as meta-progression.

The frog has a group of friends it made along the way, battleing against frightening foes that might turn the current adventurous dream into a nightmare if the frog looses. Nobody dies, it is a cute uwu game, when hp reaches 0 the creature flees.

Friend creatures are recruited among the foes via an **anger system**: The foes are angry (that is why they want to fight), and lowering their anger to 0 makes them ask you to become friends (and upon answering "yes" (without any cost) they immediately join your team). Hitting a foe increments their anger, giving them food decrements it, defeating a foe increments the anger of all the others, etc.

## Kind of creatures we encounter

- Animals that frogs might encouter irl like insects, rodents, hedgehogs, birbs, sneks, bunny, etc.
- Maybe even plants, rocks and stuff that could be more like tank/support.
- Dreamy phantasmagoric stuff like geometrical shapes, incarnations of concepts, and other wierd stuff ^^.

All the enemies shall be friendable, so their mechanics shall be designed in a way that could keep working if they ever become friends.

## Status effects

- *Food* (with counter) (icon could be a carrot): Almost always a good thing for the creature (not sure what it does yet). Giving food to foes to make friends with them is thus a cool tradeoff as it helps them while they are still your foes.
- *Magic* (with counter) (icon is a star): Every creature reacts to it in its own "implementation detail" way. It could be nice if it was always a kinda good~neutral thing with a tradeoff, making it more situational.
- *Phantasmagoric* (with counter): Good, and better with higer numbers, until the number is above a certain threshold and them pouf: the creature "vanishes" as it becomes an abstract dreamy concept. It could be intresting if this could remove the creature card from the deck permanently (for the current game) and turn it into a permanent (for the current game) effect or something (somewhat good when a friend becomes abstract, and somewhat bad when it happens to a foe) (like, the creature wans't defeated, it just became too abstract to be a card, but its idea is still around). Not something gamebreaking each time, more like Monster Train artifacts stuff but weaker. Every creature has its own good and bad permanent effects ("implementation detail" way). Ooooh i feel like this is a very nice idea!!
- *Night* (with counter) (icon could be a moon cresent): Bad in some way (not sure what it does yet).
- Scared (maybe with counter?): Bad in some way (not sure what it does yet).
- Scary (maybe with counter?): Works with the "scared" status effect in some way (not sure what it does yet).

## Mechanics

Not sure about stuff here, these are just ideas.

Creatures are on a 1D line, friends on the left and foes on the right.
Not sure how they move around tho ><, maybe one move per turn max?

No mana/cost to play cards! Maybe doing a one-card-per-turn Wildfrost style, or something else, but no mana/cost!

There could be "spell" cards that do something when played (in addition to classic creature cards that can be placed on the battlefield). Maybe "spell" or "item" is a bad name here, idk. There could be "sells" that target a creature to do something to it (like give it 2 food or something), and there could be "spells" that just apply some effect to the whole battlefield (like "fwog wants to pee" card that makes rain fall (we could play with some ideas like that to use the fact that this is supposed to be a dream)).

There shall be no limit to the number of friends! Maybe we sould find something to make it so that having too many friends is kinda bad but if we do that it must not feel like an arbitrary limit, instead it could be that due to some emergent mechanics having too many friends (creature cards in the deck) hinders something important idk.

## Action cycle mechanic!!

Every creature has a "cycle" that is a cyclic sequence of "actions". Most actions are null but some will make the creature do stuff (like attacking or whatever).
On every turn, all creatures act (unless debuff or whatever) which consists in doing the action at the top of their cycle and then sending that action at the bottom of their cycle (so that it cycles through all the actions, one action per turn).

This generalizes the counter mechanic in Wildfrost: a cycle "null, null, null, attack" is like a counter at 4/4. This is more general though, there is more possibilities of mechanics and stuff to tinker around these cycles ^^.
How about a card/creature that can remove actions from the cycles of others (can be good or bad or situational depending on what actions are removed), or insert null actions (bad), or insert complex actions (depends ^^), or take an action from one creature and insert it in the cycle of an other (depends ^^), etc. So many stuff to do here!!!
Ooooh this is a good idea!

And a creature could have more than just one kind of non-null actions, like the cycle "null, attack at 1, attack at 3, null, defend 1".

We could display this stack of actions (cycle, stack, queue, whatever xd) on the right of each creature's card, with the top action being the next thing that the creature will do.
Some actions might be very complex (needing multiple paragraphs to explain with text what it consists in) and most actions will be simple (like the null action "do nothing", or stuff like "attack at N damages" or "do X at N" (with X being whatever stuff that is common enough to have a name and that can depend on some constant); so, we could display actions as short names with an icon and an optional number (a summary of the action) and only display detailed explanations on hover. 
