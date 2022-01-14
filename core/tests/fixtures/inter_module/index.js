import { foo as foo2, bar } from "./foo"

const foo = () => {
   console.log("I'm the real foo function")
}

(function() {
   foo();
   foo2();
   bar();
})();