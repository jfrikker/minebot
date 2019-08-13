import libminebot

c = libminebot.connect_local("bilbo")
position = c.get_my_position()
print "Position: ", position
position = (259.5, 63.0, 27.5)

# wood = c.find_block_ids_within(17, position, 50)[0]
# print "Wood at", wood

# path = c.find_path_to(position, wood)
# print path

matcher = libminebot.EventMatcher()

def listen_for_ticks(c, matcher, ticks):
    end = c.current_tick() + ticks
    matcher = libminebot.EventMatcher(matcher)
    matcher.listen_tick(end)
    return c.listen_for(matcher)

# i = 0
# while True:
#     while i < len(path):
#         event = listen_for_ticks(c, matcher, 1)
#         if event.is_tick_reached():
#             print i
#             c.teleport_to((path[i][0] + 0.5, path[i][1], path[i][2] + 0.5))
#             i += 1
#     i -= 1
#     while i >= 0:
#         event = listen_for_ticks(c, matcher, 1)
#         if event.is_tick_reached():
#             print i
#             c.teleport_to((path[i][0] + 0.5, path[i][1], path[i][2] + 0.5))
#             i -= 1

# c.teleport_to((261, 72, 2))
c.enable_move(True)

angle = 0
while True:
    event = listen_for_ticks(c, matcher, 10)
    if event.is_tick_reached():
        c.set_my_yaw(angle)
        angle = (angle + 10) % 360
