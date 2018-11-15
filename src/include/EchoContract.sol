pragma solidity ^0.4.24;


// simple echo contract for debugging/example purposes
contract EchoContract {

    // echoes arbitrary bytes    
    function echo(bytes data) public pure returns(bytes) {
        return(data);
    }
}

