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

matchers = libminebot.EventMatchers()
matchers.listen_health()

while True:
    event = c.listen_for(matchers)
    if event.is_health():
        old = event.health_old()
        new = event.health_new()
        if old > new:
            c.say("Ouch! My health dropped from %.1f to %.1f" % (old, new))