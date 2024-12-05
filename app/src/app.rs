use leptos::{component, view, IntoView};
use leptos_meta::Meta;
use leptos_meta::{provide_meta_context, Stylesheet};

use crate::screens::router::Router;

#[component]
pub fn app() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/style.css"/>
        <script>
            "
            let pendingEvents = [];
            function sendGameEvent(event) {
                if (typeof globalThis['innerSendGameEvent'] !== 'function') {
                    console.warn('pushing message before module initialised');
                    console.warn(event)
                    pendingEvents.push(event);
                } else {
                    globalThis['innerSendGameEvent'](event);
                }
            }
            
            "
        </script>
        <script>
            // Insert hack to make sound autoplay on Chrome as soon as the user interacts with the tab:
            // https://developers.google.com/web/updates/2018/11/web-audio-autoplay#moving-forward

            // the following function keeps track of all AudioContexts and resumes them on the first user
            // interaction with the page. If the function is called and all contexts are already running,
            // it will remove itself from all event listeners.
            (function () {
              // An array of all contexts to resume on the page
              const audioContextList = [];

              // An array of various user interaction events we should listen for
              const userInputEventNames = [
                "click",
                "contextmenu",
                "auxclick",
                "dblclick",
                "mousedown",
                "mouseup",
                "pointerup",
                "touchend",
                "keydown",
                "keyup",
              ];

              // A proxy object to intercept AudioContexts and
              // add them to the array for tracking and resuming later
              self.AudioContext = new Proxy(self.AudioContext, {
                construct(target, args) {
                  const result = new target(...args);
                  console.log("intercepting audio context");
                  audioContextList.push(result);
                  return result;
                },
              });

              // To resume all AudioContexts being tracked
              function resumeAllContexts(_event) {
                let count = 0;

                audioContextList.forEach((context) => {
                  if (context.state !== "running") {
                    context.resume();
                  } else {
                    count++;
                  }
                });

                // If all the AudioContexts have now resumed then we unbind all
                // the event listeners from the page to prevent unnecessary resume attempts
                // Checking count > 0 ensures that the user interaction happens AFTER the game started up
                if (count > 0 && count === audioContextList.length) {
                  console.log("unbinding audio context listeners");
                  userInputEventNames.forEach((eventName) => {
                    document.removeEventListener(eventName, resumeAllContexts);
                  });
                }
              }

              // We bind the resume function for each user interaction
              // event on the page
              userInputEventNames.forEach((eventName) => {
                document.addEventListener(eventName, resumeAllContexts);
              });
            })();
        // An array of all contexts to resume on the page

        // An array of various user interaction events we should listen for

        // A proxy object to intercept AudioContexts and
        // add them to the array for tracking and resuming later

        // To resume all AudioContexts being tracked

        // If all the AudioContexts have now resumed then we unbind all
        // the event listeners from the page to prevent unnecessary resume attempts
        // Checking count > 0 ensures that the user interaction happens AFTER the game started up

        // We bind the resume function for each user interaction
        // event on the page
        // An array of all contexts to resume on the page

        // An array of various user interaction events we should listen for

        // A proxy object to intercept AudioContexts and
        // add them to the array for tracking and resuming later

        // To resume all AudioContexts being tracked

        // If all the AudioContexts have now resumed then we unbind all
        // the event listeners from the page to prevent unnecessary resume attempts
        // Checking count > 0 ensures that the user interaction happens AFTER the game started up

        // We bind the resume function for each user interaction
        // event on the page
        </script>
        <Meta
            name="viewport"
            content="width=device-width, initial-scale=1, maximum-scale=1, minimum-scale=1, user-scalable=no"
        />
        <div class="background-image"></div>
        <div class="root-container" on:click=|_| {}>
            <Router/>
        </div>
    }
}
