import libminebot

c = libminebot.connect_local("bilbo")
print "Health: ", c.get_health()
print "Food: ", c.get_food()
position = c.get_my_position()
print "Position: ", position

(x, y, z) = position
below = (x, y-1, z)

bs = c.get_block_state_at(below)
print "The block below me is", bs.get_id()

wood = c.find_block_ids_within(17, position, 50)[0]
print "Wood at", wood

print c.find_path_to(position, wood)

print "Logged in players", c.get_player_names()

matchers = libminebot.EventMatchers()
matchers.listen_health()
matchers.listen_player_list()
matchers.listen_chat()

while True:
    event = c.listen_for(matchers)
    print "event!"
    if event.is_health():
        old = event.health_old()
        new = event.health_new()
        if old > new:
            c.say("Ouch! My health dropped from %.1f to %.1f" % (old, new))
    elif len(event.players_joined()) > 0:
        for name in event.players_joined():
            c.say("Hi " + name + "!")
    elif len(event.players_left()) > 0:
        for name in event.players_left():
            c.say("Bye " + name + "!")
    elif event.is_chat():
        print "%s: %s" % (event.chat_player(), event.chat_message())