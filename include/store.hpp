/*

Based on: 

- https://github.com/sutbult/redux-cpp
- https://github.com/sumory/mas

Copyright (c) 2016 Samuel Utbult
Copyright (c) 2019 sumory.wu
Copyright (c) 2021 Michael Williams

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

See also: 
*/

#include <functional>
#include <vector>

template <typename STATE_T, typename ACTION_T>
class Store {
	public:
		typedef std::function<STATE_T(STATE_T, ACTION_T)> Reducer;
		typedef std::function<void(STATE_T)> Subscriber;

		Store(Reducer, STATE_T);
		void subscribe(Subscriber);
		void dispatch(ACTION_T);
		STATE_T getState();

	private:
		Reducer reducer;
		STATE_T state;
		std::vector<Subscriber> subscribers;
};

template <typename STATE_T, typename ACTION_T>
Store<STATE_T, ACTION_T>::Store(Reducer reducer, STATE_T initialState) :
	reducer(reducer),
	state(initialState) {
}

template <typename STATE_T, typename ACTION_T>
void Store<STATE_T, ACTION_T>::subscribe(Subscriber subscriber) {
	subscribers.push_back(std::move(subscriber));
}

template <typename STATE_T, typename ACTION_T>
void Store<STATE_T, ACTION_T>::dispatch(ACTION_T action) {
	state = reducer(state, action);

	for(uint i = 0; i < subscribers.size(); i++) {
		subscribers[i](state);
	}
}

template <typename STATE_T, typename ACTION_T>
STATE_T Store<STATE_T, ACTION_T>::getState() {
	return state;
}