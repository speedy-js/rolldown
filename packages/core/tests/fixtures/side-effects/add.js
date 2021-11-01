export const noUsed = () => {
  return `I'm no used function`
}

export const add = (s1, s2) => {
  return s1 + s2
}

add.name = 'Function(add)'

console.log(add.name)