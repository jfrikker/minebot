import libminebot

c = libminebot.connect_local("bilbo")
print "Health: ", c.health()
print "Food: ", c.food()
print "Position: ", c.my_position()

matchers = libminebot.EventMatchers()
matchers.listen_health()

while True:
    event = c.listen_for(matchers)
    if event.is_health():
        old = event.health_old()
        new = event.health_new()
        if old > new:
            c.say("Ouch! My health dropped from %.1f to %.1f" % (old, new))