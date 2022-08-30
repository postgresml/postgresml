/******************************************************************************
 * Copyright (c) 2011, Duane Merrill.  All rights reserved.
 * Copyright (c) 2011-2016, NVIDIA CORPORATION.  All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *     * Redistributions of source code must retain the above copyright
 *       notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above copyright
 *       notice, this list of conditions and the following disclaimer in the
 *       documentation and/or other materials provided with the distribution.
 *     * Neither the name of the NVIDIA CORPORATION nor the
 *       names of its contributors may be used to endorse or promote products
 *       derived from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL NVIDIA CORPORATION BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 ******************************************************************************/

/**
 * \file
 * AgentRadixSortDownsweep implements a stateful abstraction of CUDA thread blocks for participating in device-wide radix sort downsweep .
 */


#pragma once

#include "../thread/thread_load.cuh"
#include "../block/block_load.cuh"
#include "../block/block_store.cuh"
#include "../block/block_radix_rank.cuh"
#include "../block/block_exchange.cuh"
#include "../util_type.cuh"
#include "../iterator/cache_modified_input_iterator.cuh"
#include "../util_namespace.cuh"

/// Optional outer namespace(s)
CUB_NS_PREFIX

/// CUB namespace
namespace cub {


/******************************************************************************
 * Tuning policy types
 ******************************************************************************/

/**
 * Types of scattering strategies
 */
enum RadixSortScatterAlgorithm
{
    RADIX_SORT_SCATTER_DIRECT,      ///< Scatter directly from registers to global bins
    RADIX_SORT_SCATTER_TWO_PHASE,   ///< First scatter from registers into shared memory bins, then into global bins
};


/**
 * Parameterizable tuning policy type for AgentRadixSortDownsweep
 */
template <
    int                         _BLOCK_THREADS,             ///< Threads per thread block
    int                         _ITEMS_PER_THREAD,          ///< Items per thread (per tile of input)
    BlockLoadAlgorithm          _LOAD_ALGORITHM,            ///< The BlockLoad algorithm to use
    CacheLoadModifier           _LOAD_MODIFIER,             ///< Cache load modifier for reading keys (and values)
    bool                        _MEMOIZE_OUTER_SCAN,        ///< Whether or not to buffer outer raking scan partials to incur fewer shared memory reads at the expense of higher register pressure.  See BlockScanAlgorithm::BLOCK_SCAN_RAKING_MEMOIZE for more details.
    BlockScanAlgorithm          _INNER_SCAN_ALGORITHM,      ///< The BlockScan algorithm algorithm to use
    RadixSortScatterAlgorithm   _SCATTER_ALGORITHM,         ///< The scattering strategy to use
    int                         _RADIX_BITS>                ///< The number of radix bits, i.e., log2(bins)
struct AgentRadixSortDownsweepPolicy
{
    enum
    {
        BLOCK_THREADS           = _BLOCK_THREADS,           ///< Threads per thread block
        ITEMS_PER_THREAD        = _ITEMS_PER_THREAD,        ///< Items per thread (per tile of input)
        RADIX_BITS              = _RADIX_BITS,              ///< The number of radix bits, i.e., log2(bins)
        MEMOIZE_OUTER_SCAN      = _MEMOIZE_OUTER_SCAN,      ///< Whether or not to buffer outer raking scan partials to incur fewer shared memory reads at the expense of higher register pressure.  See BlockScanAlgorithm::BLOCK_SCAN_RAKING_MEMOIZE for more details.
    };

    static const BlockLoadAlgorithm         LOAD_ALGORITHM          = _LOAD_ALGORITHM;          ///< The BlockLoad algorithm to use
    static const CacheLoadModifier          LOAD_MODIFIER           = _LOAD_MODIFIER;           ///< Cache load modifier for reading keys (and values)
    static const BlockScanAlgorithm         INNER_SCAN_ALGORITHM    = _INNER_SCAN_ALGORITHM;    ///< The BlockScan algorithm algorithm to use
    static const RadixSortScatterAlgorithm  SCATTER_ALGORITHM       = _SCATTER_ALGORITHM;       ///< The scattering strategy to use
};


/******************************************************************************
 * Thread block abstractions
 ******************************************************************************/

/**
 * \brief AgentRadixSortDownsweep implements a stateful abstraction of CUDA thread blocks for participating in device-wide radix sort downsweep .
 */
template <
    typename AgentRadixSortDownsweepPolicy,     ///< Parameterized AgentRadixSortDownsweepPolicy tuning policy type
    bool     IS_DESCENDING,                        ///< Whether or not the sorted-order is high-to-low
    typename KeyT,                              ///< KeyT type
    typename ValueT,                            ///< ValueT type
    typename OffsetT>                           ///< Signed integer type for global offsets
struct AgentRadixSortDownsweep
{
    //---------------------------------------------------------------------
    // Type definitions and constants
    //---------------------------------------------------------------------

    // Appropriate unsigned-bits representation of KeyT
    typedef typename Traits<KeyT>::UnsignedBits UnsignedBits;

    static const UnsignedBits LOWEST_KEY = Traits<KeyT>::LOWEST_KEY;
    static const UnsignedBits MAX_KEY = Traits<KeyT>::MAX_KEY;

    static const BlockLoadAlgorithm         LOAD_ALGORITHM          = AgentRadixSortDownsweepPolicy::LOAD_ALGORITHM;
    static const CacheLoadModifier          LOAD_MODIFIER           = AgentRadixSortDownsweepPolicy::LOAD_MODIFIER;
    static const BlockScanAlgorithm         INNER_SCAN_ALGORITHM    = AgentRadixSortDownsweepPolicy::INNER_SCAN_ALGORITHM;
    static const RadixSortScatterAlgorithm  SCATTER_ALGORITHM       = AgentRadixSortDownsweepPolicy::SCATTER_ALGORITHM;

    enum
    {
        BLOCK_THREADS           = AgentRadixSortDownsweepPolicy::BLOCK_THREADS,
        ITEMS_PER_THREAD        = AgentRadixSortDownsweepPolicy::ITEMS_PER_THREAD,
        RADIX_BITS              = AgentRadixSortDownsweepPolicy::RADIX_BITS,
        MEMOIZE_OUTER_SCAN      = AgentRadixSortDownsweepPolicy::MEMOIZE_OUTER_SCAN,
        TILE_ITEMS              = BLOCK_THREADS * ITEMS_PER_THREAD,

        RADIX_DIGITS            = 1 << RADIX_BITS,
        KEYS_ONLY               = Equals<ValueT, NullType>::VALUE,

        WARP_THREADS            = CUB_PTX_LOG_WARP_THREADS,
        WARPS                   = (BLOCK_THREADS + WARP_THREADS - 1) / WARP_THREADS,

        BYTES_PER_SIZET         = sizeof(OffsetT),
        LOG_BYTES_PER_SIZET     = Log2<BYTES_PER_SIZET>::VALUE,

        LOG_SMEM_BANKS          = CUB_PTX_LOG_SMEM_BANKS,
        SMEM_BANKS              = 1 << LOG_SMEM_BANKS,

        DIGITS_PER_SCATTER_PASS = BLOCK_THREADS / SMEM_BANKS,
        SCATTER_PASSES          = RADIX_DIGITS / DIGITS_PER_SCATTER_PASS,

        LOG_STORE_TXN_THREADS   = LOG_SMEM_BANKS,
        STORE_TXN_THREADS       = 1 << LOG_STORE_TXN_THREADS,
    };

    // Input iterator wrapper type (for applying cache modifier)s
    typedef CacheModifiedInputIterator<LOAD_MODIFIER, UnsignedBits, OffsetT>    KeysItr;
    typedef CacheModifiedInputIterator<LOAD_MODIFIER, ValueT, OffsetT>          ValuesItr;

    // BlockRadixRank type
    typedef BlockRadixRank<
        BLOCK_THREADS,
        RADIX_BITS,
        IS_DESCENDING,
        MEMOIZE_OUTER_SCAN,
        INNER_SCAN_ALGORITHM> BlockRadixRank;

    // BlockLoad type (keys)
    typedef BlockLoad<
        UnsignedBits,
        BLOCK_THREADS,
        ITEMS_PER_THREAD,
        LOAD_ALGORITHM> BlockLoadKeys;

    // BlockLoad type (values)
    typedef BlockLoad<
        ValueT,
        BLOCK_THREADS,
        ITEMS_PER_THREAD,
        LOAD_ALGORITHM> BlockLoadValues;

    // BlockExchange type (keys)
    typedef BlockExchange<
        UnsignedBits,
        BLOCK_THREADS,
        ITEMS_PER_THREAD> BlockExchangeKeys;

    // BlockExchange type (values)
    typedef BlockExchange<
        ValueT,
        BLOCK_THREADS,
        ITEMS_PER_THREAD> BlockExchangeValues;


    /**
     * Shared memory storage layout
     */
    union __align__(16) _TempStorage
    {
        typename BlockLoadKeys::TempStorage         load_keys;
        typename BlockRadixRank::TempStorage        ranking;
        typename BlockLoadValues::TempStorage       load_values;
        typename BlockExchangeValues::TempStorage   exchange_values;

        OffsetT     exclusive_digit_prefix[RADIX_DIGITS];

        struct
        {
            typename BlockExchangeKeys::TempStorage     exchange_keys;
            OffsetT     relative_bin_offsets[RADIX_DIGITS + 1];
        };

    };


    /// Alias wrapper allowing storage to be unioned
    struct TempStorage : Uninitialized<_TempStorage> {};


    //---------------------------------------------------------------------
    // Thread fields
    //---------------------------------------------------------------------

    // Shared storage for this CTA
    _TempStorage    &temp_storage;

    // Input and output device pointers
    KeysItr         d_keys_in;
    ValuesItr       d_values_in;
    UnsignedBits    *d_keys_out;
    ValueT          *d_values_out;

    // The global scatter base offset for each digit (valid in the first RADIX_DIGITS threads)
    OffsetT         bin_offset;

    // The least-significant bit position of the current digit to extract
    int             current_bit;

    // Number of bits in current digit
    int             num_bits;

    // Whether to short-cirucit
    int             short_circuit;

    //---------------------------------------------------------------------
    // Utility methods
    //---------------------------------------------------------------------

    /**
     * Scatter ranked keys directly to device-accessible memory
     */
    template <bool FULL_TILE>
    __device__ __forceinline__ void ScatterKeys(
        UnsignedBits                            (&twiddled_keys)[ITEMS_PER_THREAD],
        OffsetT                                 (&relative_bin_offsets)[ITEMS_PER_THREAD],
        int                                     (&ranks)[ITEMS_PER_THREAD],
        OffsetT                                 valid_items,
        Int2Type<RADIX_SORT_SCATTER_DIRECT>     /*scatter_algorithm*/)
    {
        #pragma unroll
        for (int ITEM = 0; ITEM < ITEMS_PER_THREAD; ++ITEM)
        {
            UnsignedBits digit          = BFE(twiddled_keys[ITEM], current_bit, num_bits);
            relative_bin_offsets[ITEM]  = temp_storage.relative_bin_offsets[digit];

            // Un-twiddle
            UnsignedBits key            = Traits<KeyT>::TwiddleOut(twiddled_keys[ITEM]);

            if (FULL_TILE || (ranks[ITEM] < valid_items))
            {
                d_keys_out[relative_bin_offsets[ITEM] + ranks[ITEM]] = key;
            }
        }
    }


    /**
     * Scatter ranked keys through shared memory, then to device-accessible memory
     */
    template <bool FULL_TILE>
    __device__ __forceinline__ void ScatterKeys(
        UnsignedBits                            (&twiddled_keys)[ITEMS_PER_THREAD],
        OffsetT                                 (&relative_bin_offsets)[ITEMS_PER_THREAD],
        int                                     (&ranks)[ITEMS_PER_THREAD],
        OffsetT                                 valid_items,
        Int2Type<RADIX_SORT_SCATTER_TWO_PHASE>  /*scatter_algorithm*/)
    {
        UnsignedBits *smem = reinterpret_cast<UnsignedBits*>(&temp_storage.exchange_keys);

        #pragma unroll
        for (int ITEM = 0; ITEM < ITEMS_PER_THREAD; ++ITEM)
        {
            smem[ranks[ITEM]] = twiddled_keys[ITEM];
        }

        CTA_SYNC();

        #pragma unroll
        for (int ITEM = 0; ITEM < ITEMS_PER_THREAD; ++ITEM)
        {
            UnsignedBits key = smem[threadIdx.x + (ITEM * BLOCK_THREADS)];

            UnsignedBits digit = BFE(key, current_bit, num_bits);

            relative_bin_offsets[ITEM] = temp_storage.relative_bin_offsets[digit];

            // Un-twiddle
            key = Traits<KeyT>::TwiddleOut(key);

            if (FULL_TILE || 
                (static_cast<OffsetT>(threadIdx.x + (ITEM * BLOCK_THREADS)) < valid_items))
            {
                d_keys_out[relative_bin_offsets[ITEM] + threadIdx.x + (ITEM * BLOCK_THREADS)] = key;
            }
        }
    }



    /**
     * Scatter ranked values directly to device-accessible memory
     */
    template <bool FULL_TILE>
    __device__ __forceinline__ void ScatterValues(
        ValueT                                  (&values)[ITEMS_PER_THREAD],
        OffsetT                                 (&relative_bin_offsets)[ITEMS_PER_THREAD],
        int                                     (&ranks)[ITEMS_PER_THREAD],
        OffsetT                                 valid_items,
        Int2Type<RADIX_SORT_SCATTER_DIRECT>     /*scatter_algorithm*/)
    {
        #pragma unroll
        for (int ITEM = 0; ITEM < ITEMS_PER_THREAD; ++ITEM)
        {
            if (FULL_TILE || (ranks[ITEM] < valid_items))
            {
                d_values_out[relative_bin_offsets[ITEM] + ranks[ITEM]] = values[ITEM];
            }
        }
    }


    /**
     * Scatter ranked values through shared memory, then to device-accessible memory
     */
    template <bool FULL_TILE>
    __device__ __forceinline__ void ScatterValues(
        ValueT                                  (&values)[ITEMS_PER_THREAD],
        OffsetT                                 (&relative_bin_offsets)[ITEMS_PER_THREAD],
        int                                     (&ranks)[ITEMS_PER_THREAD],
        OffsetT                                 valid_items,
        Int2Type<RADIX_SORT_SCATTER_TWO_PHASE>  /*scatter_algorithm*/)
    {
        CTA_SYNC();

        ValueT *smem = reinterpret_cast<ValueT*>(&temp_storage.exchange_values);

        #pragma unroll
        for (int ITEM = 0; ITEM < ITEMS_PER_THREAD; ++ITEM)
        {
            smem[ranks[ITEM]] = values[ITEM];
        }

        CTA_SYNC();

        #pragma unroll
        for (int ITEM = 0; ITEM < ITEMS_PER_THREAD; ++ITEM)
        {
            ValueT value = smem[threadIdx.x + (ITEM * BLOCK_THREADS)];

            if (FULL_TILE || 
                (static_cast<OffsetT>(threadIdx.x + (ITEM * BLOCK_THREADS)) < valid_items))
            {
                d_values_out[relative_bin_offsets[ITEM] + threadIdx.x + (ITEM * BLOCK_THREADS)] = value;
            }
        }
    }


    /**
     * Load a tile of items (specialized for full tile)
     */
    template <typename BlockLoadT, typename T, typename InputIteratorT>
    __device__ __forceinline__ void LoadItems(
        BlockLoadT      &block_loader, 
        T               (&items)[ITEMS_PER_THREAD],
        InputIteratorT  d_in,
        OffsetT         /*valid_items*/,
        Int2Type<true>  /*is_full_tile*/)
    {
        block_loader.Load(d_in, items);
    }


    /**
     * Load a tile of items (specialized for full tile)
     */
    template <typename BlockLoadT, typename T, typename InputIteratorT>
    __device__ __forceinline__ void LoadItems(
        BlockLoadT      &block_loader,
        T               (&items)[ITEMS_PER_THREAD],
        InputIteratorT  d_in,
        OffsetT         /*valid_items*/,
        T               /*oob_item*/,
        Int2Type<true>  /*is_full_tile*/)
    {
        block_loader.Load(d_in, items);
    }


    /**
     * Load a tile of items (specialized for partial tile)
     */
    template <typename BlockLoadT, typename T, typename InputIteratorT>
    __device__ __forceinline__ void LoadItems(
        BlockLoadT      &block_loader, 
        T               (&items)[ITEMS_PER_THREAD],
        InputIteratorT  d_in,
        OffsetT         valid_items,
        Int2Type<false> /*is_full_tile*/)
    {
        block_loader.Load(d_in, items, valid_items);
    }

    /**
     * Load a tile of items (specialized for partial tile)
     */
    template <typename BlockLoadT, typename T, typename InputIteratorT>
    __device__ __forceinline__ void LoadItems(
        BlockLoadT      &block_loader,
        T               (&items)[ITEMS_PER_THREAD],
        InputIteratorT  d_in,
        OffsetT         valid_items,
        T               oob_item,
        Int2Type<false> /*is_full_tile*/)
    {
        block_loader.Load(d_in, items, valid_items, oob_item);
    }


    /**
     * Truck along associated values
     */
    template <bool FULL_TILE>
    __device__ __forceinline__ void GatherScatterValues(
        OffsetT         (&relative_bin_offsets)[ITEMS_PER_THREAD],
        int             (&ranks)[ITEMS_PER_THREAD],
        OffsetT         block_offset,
        OffsetT         valid_items,
        Int2Type<false> /*is_keys_only*/)
    {
        CTA_SYNC();

        ValueT values[ITEMS_PER_THREAD];

        BlockLoadValues loader(temp_storage.load_values);
        LoadItems(
            loader,
            values,
            d_values_in + block_offset,
            valid_items,
            Int2Type<FULL_TILE>());

        ScatterValues<FULL_TILE>(
            values,
            relative_bin_offsets,
            ranks,
            valid_items,
            Int2Type<SCATTER_ALGORITHM>());
    }


    /**
     * Truck along associated values (specialized for key-only sorting)
     */
    template <bool FULL_TILE>
    __device__ __forceinline__ void GatherScatterValues(
        OffsetT         (&/*relative_bin_offsets*/)[ITEMS_PER_THREAD],
        int             (&/*ranks*/)[ITEMS_PER_THREAD],
        OffsetT         /*block_offset*/,
        OffsetT         /*valid_items*/,
        Int2Type<true>  /*is_keys_only*/)
    {}


    /**
     * Process tile
     */
    template <bool FULL_TILE>
    __device__ __forceinline__ void ProcessTile(
        OffsetT block_offset,
        const OffsetT &valid_items = TILE_ITEMS)
    {
        // Per-thread tile data
        UnsignedBits    keys[ITEMS_PER_THREAD];                     // Keys
        UnsignedBits    twiddled_keys[ITEMS_PER_THREAD];            // Twiddled keys
        int             ranks[ITEMS_PER_THREAD];                    // For each key, the local rank within the CTA
        OffsetT         relative_bin_offsets[ITEMS_PER_THREAD];     // For each key, the global scatter base offset of the corresponding digit

        // Assign default (min/max) value to all keys
        UnsignedBits default_key = (IS_DESCENDING) ? LOWEST_KEY : MAX_KEY;

        // Load tile of keys
        BlockLoadKeys loader(temp_storage.load_keys);
        LoadItems(
            loader,
            keys,
            d_keys_in + block_offset,
            valid_items, 
            default_key,
            Int2Type<FULL_TILE>());

        CTA_SYNC();

        // Twiddle key bits if necessary
        #pragma unroll
        for (int KEY = 0; KEY < ITEMS_PER_THREAD; KEY++)
        {
            twiddled_keys[KEY] = Traits<KeyT>::TwiddleIn(keys[KEY]);
        }

        // Rank the twiddled keys
        int exclusive_digit_prefix;
        BlockRadixRank(temp_storage.ranking).RankKeys(
            twiddled_keys,
            ranks,
            current_bit,
            num_bits,
            exclusive_digit_prefix);

        CTA_SYNC();

        // Share exclusive digit prefix
        if (threadIdx.x < RADIX_DIGITS)
        {
            // Store exclusive prefix
            temp_storage.exclusive_digit_prefix[threadIdx.x] = exclusive_digit_prefix;
        }

        CTA_SYNC();

        // Get inclusive digit prefix
        int inclusive_digit_prefix;
        if (threadIdx.x < RADIX_DIGITS)
        {
            if (IS_DESCENDING)
            {
                // Get inclusive digit prefix from exclusive prefix (higher bins come first)
                inclusive_digit_prefix = (threadIdx.x == 0) ?
                    (BLOCK_THREADS * ITEMS_PER_THREAD) :
                    temp_storage.exclusive_digit_prefix[threadIdx.x - 1];
            }
            else
            {
                // Get inclusive digit prefix from exclusive prefix (lower bins come first)
                inclusive_digit_prefix = (threadIdx.x == RADIX_DIGITS - 1) ?
                    (BLOCK_THREADS * ITEMS_PER_THREAD) :
                    temp_storage.exclusive_digit_prefix[threadIdx.x + 1];
            }
        }

        CTA_SYNC();

        // Update global scatter base offsets for each digit
        if (threadIdx.x < RADIX_DIGITS)
        {


            bin_offset -= exclusive_digit_prefix;
            temp_storage.relative_bin_offsets[threadIdx.x] = bin_offset;
            bin_offset += inclusive_digit_prefix;
        }

        CTA_SYNC();

        // Scatter keys
        ScatterKeys<FULL_TILE>(twiddled_keys, relative_bin_offsets, ranks, valid_items, Int2Type<SCATTER_ALGORITHM>());

        // Gather/scatter values
        GatherScatterValues<FULL_TILE>(relative_bin_offsets , ranks, block_offset, valid_items, Int2Type<KEYS_ONLY>());
    }

    //---------------------------------------------------------------------
    // Copy shortcut
    //---------------------------------------------------------------------

    /**
     * Copy tiles within the range of input
     */
    template <
        typename InputIteratorT,
        typename T>
    __device__ __forceinline__ void Copy(
        InputIteratorT  d_in,
        T               *d_out,
        OffsetT         block_offset,
        OffsetT         block_end)
    {
        // Simply copy the input
        while (block_offset + TILE_ITEMS <= block_end)
        {
            T items[ITEMS_PER_THREAD];

            LoadDirectStriped<BLOCK_THREADS>(threadIdx.x, d_in + block_offset, items);
            CTA_SYNC();
            StoreDirectStriped<BLOCK_THREADS>(threadIdx.x, d_out + block_offset, items);

            block_offset += TILE_ITEMS;
        }

        // Clean up last partial tile with guarded-I/O
        if (block_offset < block_end)
        {
            OffsetT valid_items = block_end - block_offset;

            T items[ITEMS_PER_THREAD];

            LoadDirectStriped<BLOCK_THREADS>(threadIdx.x, d_in + block_offset, items, valid_items);
            CTA_SYNC();
            StoreDirectStriped<BLOCK_THREADS>(threadIdx.x, d_out + block_offset, items, valid_items);
        }
    }


    /**
     * Copy tiles within the range of input (specialized for NullType)
     */
    template <typename InputIteratorT>
    __device__ __forceinline__ void Copy(
        InputIteratorT  /*d_in*/,
        NullType        * /*d_out*/,
        OffsetT         /*block_offset*/,
        OffsetT         /*block_end*/)
    {}


    //---------------------------------------------------------------------
    // Interface
    //---------------------------------------------------------------------

    /**
     * Constructor
     */
    __device__ __forceinline__ AgentRadixSortDownsweep(
        TempStorage     &temp_storage,
        OffsetT         num_items,
        OffsetT         bin_offset,
        const KeyT      *d_keys_in,
        KeyT            *d_keys_out,
        const ValueT    *d_values_in,
        ValueT          *d_values_out,
        int             current_bit,
        int             num_bits)
    :
        temp_storage(temp_storage.Alias()),
        bin_offset(bin_offset),
        d_keys_in(reinterpret_cast<const UnsignedBits*>(d_keys_in)),
        d_values_in(d_values_in),
        d_keys_out(reinterpret_cast<UnsignedBits*>(d_keys_out)),
        d_values_out(d_values_out),
        current_bit(current_bit),
        num_bits(num_bits),
        short_circuit(1)
    {
        if (threadIdx.x < RADIX_DIGITS)
        {
            // Short circuit if the histogram has only bin counts of only zeros or problem-size
            short_circuit = ((bin_offset == 0) || (bin_offset == num_items));
        }

        short_circuit = CTA_SYNC_AND(short_circuit);
    }


    /**
     * Constructor
     */
    __device__ __forceinline__ AgentRadixSortDownsweep(
        TempStorage     &temp_storage,
        OffsetT         num_items,
        OffsetT         *d_spine,
        const KeyT      *d_keys_in,
        KeyT            *d_keys_out,
        const ValueT    *d_values_in,
        ValueT          *d_values_out,
        int             current_bit,
        int             num_bits)
    :
        temp_storage(temp_storage.Alias()),
        d_keys_in(reinterpret_cast<const UnsignedBits*>(d_keys_in)),
        d_values_in(d_values_in),
        d_keys_out(reinterpret_cast<UnsignedBits*>(d_keys_out)),
        d_values_out(d_values_out),
        current_bit(current_bit),
        num_bits(num_bits),
        short_circuit(1)
    {
        // Load digit bin offsets (each of the first RADIX_DIGITS threads will load an offset for that digit)
        if (threadIdx.x < RADIX_DIGITS)
        {
            int bin_idx = (IS_DESCENDING) ?
                RADIX_DIGITS - threadIdx.x - 1 :
                threadIdx.x;

            // Short circuit if the first block's histogram has only bin counts of only zeros or problem-size
            OffsetT first_block_bin_offset = d_spine[gridDim.x * bin_idx];
            short_circuit = ((first_block_bin_offset == 0) || (first_block_bin_offset == num_items));

            // Load my block's bin offset for my bin
            bin_offset = d_spine[(gridDim.x * bin_idx) + blockIdx.x];
        }

        short_circuit = CTA_SYNC_AND(short_circuit);
    }


    /**
     * Distribute keys from a segment of input tiles.
     */
    __device__ __forceinline__ void ProcessRegion(
        OffsetT   block_offset,
        OffsetT   block_end)
    {
        if (short_circuit)
        {
            // Copy keys
            Copy(d_keys_in, d_keys_out, block_offset, block_end);

            // Copy values
            Copy(d_values_in, d_values_out, block_offset, block_end);
        }
        else
        {
            // Process full tiles of tile_items
            while (block_offset + TILE_ITEMS <= block_end)
            {
                ProcessTile<true>(block_offset);
                block_offset += TILE_ITEMS;

                CTA_SYNC();
            }

            // Clean up last partial tile with guarded-I/O
            if (block_offset < block_end)
            {
                ProcessTile<false>(block_offset, block_end - block_offset);
            }
        }
    }

};



}               // CUB namespace
CUB_NS_POSTFIX  // Optional outer namespace(s)

