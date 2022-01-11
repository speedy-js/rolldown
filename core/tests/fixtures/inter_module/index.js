import { foo as foo2 } from "./foo"

const foo = () => {
   console.log("I'm the real foo function")
}

(function() {
   foo();
   foo2();
})();