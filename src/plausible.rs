use rand::Rng;

const TEXTS: &[&str] = &[
    "monday\nwoke up late again. couldn't find my keys for like 10 minutes. \
bus was packed. had the leftover rice for lunch, still good. \
called the landlord about the boiler, left a voicemail. \
watched some tv, fell asleep on the sofa. classic.\n",

    "had a weird dream i was in my old school but all the classrooms were underwater. \
work meeting ran long. lunch was leftover pasta from yesterday still good though. \
finally fixed the squeaky floorboard in the hallway with a screw. small win. \
trying to read before bed more but keep falling asleep after 2 pages.\n",

    "wednesday. rained all day. stayed in and cleaned the bathroom which i had been \
putting off for literally three weeks. feels better now. ordered thai food. \
watched the rest of that documentary about penguins. pretty good. \
reminder to buy dish soap tomorrow, almost out.\n",

    "grocery run:\n- whole milk x2\n- eggs (free range)\n- sourdough bread\n\
- cheddar block\n- chicken thighs\n- pasta (fusilli or penne)\n- tinned tomatoes x3\n\
- garlic\n- onions\n- olive oil\n- washing up liquid\n- bin bags (large)\n- shampoo\n",

    "need to get:\n- bananas (the not-too-ripe ones)\n- greek yogurt\n- orange juice\n\
- frozen peas\n- jasmine rice\n- soy sauce\n- toilet roll (18 pack)\n\
- ibuprofen\n- toothpaste\n- stamps from the post office\n",

    "this week:\n\
[ ] book dentist appointment (overdue by 4 months)\n\
[ ] renew car insurance — expires the 23rd\n\
[ ] return library books before the fine gets worse\n\
[ ] email landlord about the bathroom window\n\
[ ] pick up prescription from boots\n\
[ ] water the plants (they look sad)\n\
[ ] reply to sarah about the weekend\n\
[x] pay rent\n\
[x] sort out recycling\n",

    "weekend:\n\
- farmers market saturday morning\n\
- finally sort out the garage (been saying this for 2 months)\n\
- call grandma\n\
- try that new ramen place on the high street\n\
- fix the drawer that keeps getting stuck\n\
- maybe go for a run if weather permits\n",

    "simple tomato pasta (serves 2):\n\
fry 1 onion + 4 cloves garlic in olive oil until soft, about 8 mins.\n\
add 1 tin crushed tomatoes, salt, pepper, dried basil.\n\
simmer 20 mins. add a pinch of sugar if too sharp.\n\
toss with cooked pasta. top with parmesan and black pepper.\n\
good with garlic bread on the side.\n",

    "lemon drizzle notes (adjustments from last time):\n\
used 180g sugar not 200 — definitely better, less sweet.\n\
baked 45 mins at 160 fan — perfect, didn't dry out.\n\
drizzle: 3 tbsp lemon juice + 4 tbsp icing sugar while still warm.\n\
next time try adding poppy seeds.\n",

    "things i keep forgetting:\n\
- the wifi password is on a sticker on the back of the router\n\
- bin day is tuesday not wednesday\n\
- gym membership renews automatically on the 1st\n\
- the trick with the door is you have to lift the handle slightly when locking\n\
- backup codes for email are in the notebook in the top drawer\n",
];

pub fn generate() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let text = TEXTS[rng.gen_range(0..TEXTS.len())];
    text.as_bytes().to_vec()
}
