from gym.envs.registration import register

register(
    id='warehouse/Warehouse-v0',
    entry_point='warehouse.mdp_envs:Warehouse',
    max_episode_steps=300,
)