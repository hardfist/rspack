// Test case for issue #12706
// Template literals with Unicode escape sequences (lone surrogates) should not panic

// This used to cause a panic because the cooked field was None
const regex = new RegExp(`\uD83C[\uDFFB-\uDFFF]`, 'g');

// Export to ensure the code is actually processed
export { regex };
