import gym
import warehouse
env = gym.make('warehouse/Warehouse-v0')

print("|S|", len(env.state_space))

print("|P|", len(env.transitions))