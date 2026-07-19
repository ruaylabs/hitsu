/** Tracks Hitsu-owned native dialogs (file pickers opened from the frontend
 *  or by the backend). Native dialogs steal window focus, which would trip
 *  the privacy screen and blank the app behind the dialog the user just
 *  opened — the screen keys off this store to skip that case, while true
 *  focus loss (switching apps) still hides content instantly. */

let depth = $state(0);

export const nativeDialog = {
  get open(): boolean {
    return depth > 0;
  },
  /** Run a task that opens a native dialog, suppressing the privacy screen
   *  for its duration. Reentrant: overlapping tasks are counted. */
  async during<T>(task: () => Promise<T>): Promise<T> {
    depth += 1;
    try {
      return await task();
    } finally {
      depth -= 1;
    }
  },
};
