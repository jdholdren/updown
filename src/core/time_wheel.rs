/// A simple implementation of a timing wheel
///
/// Just call tick and it will return anything needing to be run at
/// that time or moved up.

// The number of spokes in the wheel
const MAX_TICKS: usize = 60;

// Something insert into a slot
#[derive(PartialEq, Debug)]
struct Slotted<T> {
    val: T, // The value that's been inserted
    counter: usize, // If this reaches zero, time to run. Otherwise we decrement until it bottoms
            // out.
}

pub struct TimingWheel<T> {
    current_tick: usize,
    slots: [Vec<Slotted<T>>; MAX_TICKS],
}

impl<T> Default for TimingWheel<T> {
    fn default() -> Self {
        TimingWheel {
            current_tick: 0,
            slots: [(); MAX_TICKS].map(|_| Vec::new()),
        }
    }
}

impl<T> TimingWheel<T> {
    pub fn add(&mut self, t: T, seconds_from_now: usize) {
        let sum = self.current_tick + seconds_from_now;
        let index = sum % MAX_TICKS;
        let mut counter = seconds_from_now / MAX_TICKS; // Using integer math to determine how many groups of 60 are
                                                        // present
        if seconds_from_now % MAX_TICKS == 0 {
            counter -= 1;
        }
        self.slots[index].push(Slotted { val: t, counter });
    }

    pub fn tick(&mut self) -> Vec<T> {
        // Increment the current_tick to the next slot
        if self.current_tick == MAX_TICKS - 1 {
            self.current_tick = 0;
        } else {
            self.current_tick += 1;
        }

        // The vector of things we'll be operating on
        let slotteds = &mut self.slots[self.current_tick];
        if slotteds.is_empty() {
            return Vec::new();
        }

        // Loop through, and pop out anything that has counter: 0. Otherwise
        // decrease it and  move along.
        //
        // To avoid a copy, we'll mark everything we need to remove. Then, once we
        // no longer have a reference to the vector, we can pull the values out and return
        // them as owned.
        let mut to_remove = Vec::new();
        for (idx, slotted) in slotteds.iter_mut().enumerate() {
            if slotted.counter > 0 {
                slotted.counter -= 1;
                continue;
            }

            to_remove.push(idx);
        }

        // We'll be shifting everything left as we pull it out, so we need to traverse the
        // indices we just produced backwards
        to_remove.reverse();
        let mut done = Vec::new();
        for idx in to_remove {
            done.push(self.slots[self.current_tick].remove(idx).val);
        }

        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut tw = TimingWheel::<i32>::default();
        tw.add(1, 15);
        tw.add(2, 26);
        tw.add(3, 60);
        tw.add(4, 120);

        assert_eq!(&tw.slots[15][0], &Slotted::<i32> { val: 1, counter: 0 });
        assert_eq!(&tw.slots[26][0], &Slotted::<i32> { val: 2, counter: 0 });
        assert_eq!(&tw.slots[0][0], &Slotted::<i32> { val: 3, counter: 0 });
        assert_eq!(&tw.slots[0][1], &Slotted::<i32> { val: 4, counter: 1 });

        // Ticking should affect when we schedule
        tw.tick();
        tw.add(5, 60);
        tw.add(6, 59);

        assert_eq!(&tw.slots[1][0], &Slotted::<i32> { val: 5, counter: 0 });
        assert_eq!(&tw.slots[0][2], &Slotted::<i32> { val: 6, counter: 0 });
    }

    #[test]
    fn test_tick() {
        let mut tw = TimingWheel::<i32>::default();
        tw.add(1, 1);
        tw.add(2, 61);
        tw.add(3, 1);

        let ran = tw.tick();
        assert_eq!(tw.current_tick, 1);
        assert_eq!(ran, vec![3, 1]);
        assert_eq!(tw.slots[1], vec![Slotted { val: 2, counter: 0 }]);
    }
}
