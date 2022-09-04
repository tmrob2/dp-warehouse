from warehouse.mdp_envs.base import MOTAPEnv
from optparse import Option
from gym import spaces
import numpy as np
from typing import Optional
from gym.utils.renderer import Renderer
import json
import dp_warehouse

class Warehouse(MOTAPEnv):
    metadata = {"render_modes": ["human", "rgb_array", "single_rgb_array"], "render_fps": 4}

    def __init__(self, render_mode: Optional[str] = None, size: int=20, initial_agent_loc=(0,0)):
        super().__init__((initial_agent_loc, 0, None)) # use the super method to set the initial state
        assert render_mode is None or render_mode in self.metadata["render_modes"]
        self.render_mode = render_mode

        self.size = size # the size of the square grid
        self.window_size = 512 # the size of the pygame window

        self.state_mapping = {}
        self.reverse_state_mapping = {}

        # Observations are dictionaries with the agent's and target's locatoin
        # each location is encoded as an element  {0,..., size}^2 i.e. MultiDiscrete([size,size])

        self.observation_space = spaces.Dict({
            "a": spaces.Box(0, size - 1, shape=(2,), dtype=int),
            "c": spaces.Discrete(2),
            "r": spaces.Box(0, size - 1, shape=(2,), dtype=int),
        })

        self._rack_positions = dp_warehouse.place_racks(size, size) # is a set

        # there are four actions, corresponding to "right", "up", "left", "down"
        self.action_space = spaces.Discrete(6)

        """
        The following dictionary maps abstract actions from self.action_space to 
        the directions we will walk in if that action is taken
        I.e. 0 corresponds to right, so on..
        """
        self._action_to_direction = {
            0: (1, 0),
            1: (0, 1),
            2: (-1, 0),
            3: (0, -1)
        }

        """
        If human rendering is used, "self.window" will be a reference to the window that we draw to
        'self.clock' will be a clock that is used to ensure that the environment is rendered at the
        correct framerate in human-mode.
        """
        if self.render_mode == "human":
            import pygame

            pygame.init()
            pygame.display.init()
            self.window = pygame.display.set_mode((self.window_size, self.window_size))
            self.clock = pygame.time.Clock()

        # the following line uses the util class Renderer to gather a collection of frames
        # using a method that computes a single frame. We will define _render_frame below
        #self.renderer = Renderer(self.render_mode, self._render_frame)

        self.state_space = self._set_state_space()

    def _set_state_space(self):
        print("init state", self.init_state)
        (state_space, state_map, rev_state_map, transitions) = dp_warehouse.set_state_space(
            self.init_state,
            self.action_space.n,
            self._action_to_direction,
            self._rack_positions,
            (self.size,self.size)
        )
        self.state_mapping = state_map
        self.reverse_state_mapping = rev_state_map
        self.transitions = transitions
        return state_space

    
    def step(self, action):
        # the current state is self.state
        direction = self._action_to_direction[action]

        # we will use the step function to filter available actions

        if self.is_carrying == 0:
            # then an agent may move anywhere
            pass
        else:
            pass


    def reset(self):
        pass

        

