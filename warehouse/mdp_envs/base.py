
import gym

"""
We need to almost run two environments in parallel. There is the environment
which we will use for rendering, and the shadow environment methods which we
will use for dynamic programming. 
"""

class MOTAPEnv(gym.Env):
    def __init__(self, init_state) -> None:
        self.init_state = init_state
        self.state = init_state
        self.actions = {}

    def get_actions(self, state):
        return self.actions[state]

    def set_state(self, state):
        """
        We require a set state method because the product MDP creation will
        go through each of the reachable states and make a product with the 
        corresponding DFA state
        """
        self.state = state

    def _set_state_space(self):
        """
        The state space of the environment must be defined because MOTAP
        uses dynamic programming methods and directly constructs a 
        transition and rewards matrices. 
        """
        pass

    def _step(state, action):
        """
        We require a private state which compute all the transitions for a 
        state and some action
        """
        pass